import { xchacha20poly1305 } from '@noble/ciphers/chacha';
import { equalBytes } from '@noble/ciphers/utils';
import { kmac256 } from '@noble/hashes/sha3-addons';
import { toBase64Url } from './base64';
import { getRandomBytes } from './random';

const KEY_SIZE = 32; // bytes, considered quantum-safe
const NONCE_SIZE = 24; // bytes for XChaCha20 (different from ChaCha20's 12)
const ROBUSTNESS_TAG_SIZE = 32; // bytes

/**
 * Encrypts a message using XChaCha20-Poly1305 with Associated Data and
 * prepends a 32-byte KMAC-256 robustness tag over `nonce || ciphertext || AAD`.
 *
 * - If `key` or `nonce` are omitted, they are securely generated.
 * - The returned `ciphertext` layout is: `robustnessTag(32) || aeadCiphertext`.
 * - `additionalData` (AAD) must be provided unchanged for decryption.
 *
 * @param message Plaintext as UTF-8 string or bytes.
 * @param additionalData Associated Data (AAD) bound to the encryption.
 * @param key Optional 32-byte key. If omitted, a random key is generated.
 * @param nonce Optional 24-byte nonce. If omitted, a random nonce is generated.
 * @returns Object containing `ciphertext` (with robustness tag prefix), the `key`, and the `nonce` used.
 * @throws Error if a provided `key` is not exactly 32 bytes.
 */
export function encryptMessage(
  message: string | Uint8Array,
  additionalData: string,
  key?: Uint8Array,
  nonce?: Uint8Array,
): {
  ciphertext: Uint8Array;
  key: Uint8Array;
  nonce: Uint8Array;
} {
  const encryptionKey = key ?? getRandomBytes(KEY_SIZE);
  const encryptionNonce = nonce ?? getRandomBytes(NONCE_SIZE);

  if (!(encryptionKey instanceof Uint8Array) || encryptionKey.length !== KEY_SIZE) {
    throw new Error(`${KEY_SIZE}-byte key is required for XChaCha20-Poly1305`);
  }

  const messageBytes =
    typeof message === 'string' ? new TextEncoder().encode(message) : message;
  const additionalDataBytes = new TextEncoder().encode(additionalData);

  const cipher = xchacha20poly1305(encryptionKey, encryptionNonce, additionalDataBytes);
  const ciphertext = cipher.encrypt(messageBytes);

  // create robustness tag using KMAC-256 (different crypto primitives reduce risk)
  const tagInput =
    toBase64Url(encryptionNonce) + toBase64Url(ciphertext) + additionalData;
  const tagInputBytes = new TextEncoder().encode(tagInput);
  const robustnessTag = kmac256(encryptionKey, tagInputBytes, {
    dkLen: ROBUSTNESS_TAG_SIZE,
    personalization: new TextEncoder().encode('RobustnessTag-v1'),
  });

  // combine ciphertext with robustness tag
  const combinedCiphertext = new Uint8Array(robustnessTag.length + ciphertext.length);
  combinedCiphertext.set(robustnessTag);
  combinedCiphertext.set(ciphertext, robustnessTag.length);

  return {
    ciphertext: combinedCiphertext,
    key: encryptionKey,
    nonce: encryptionNonce,
  };
}

/**
 * Decrypts a ciphertext produced by {@link encryptMessage}.
 *
 * - Verifies the 32-byte KMAC-256 robustness tag over `nonce || ciphertext || AAD`.
 * - If verification succeeds, decrypts with XChaCha20-Poly1305 using the provided `key`, `nonce` and `additionalData`.
 *
 * @param combinedCiphertext Input as `robustnessTag(32) || aeadCiphertext`.
 * @param additionalData Associated Data (AAD) that must match the value used at encryption.
 * @param key 32-byte key (`Uint8Array`).
 * @param nonce 24-byte nonce (`Uint8Array`).
 * @returns Decrypted plaintext as a UTF-8 string.
 * @throws Error when the robustness tag does not match (tampering or wrong inputs).
 */
export function decryptMessage(
  combinedCiphertext: Uint8Array,
  additionalData: string,
  key: Uint8Array,
  nonce: Uint8Array,
): string {
  const additionalDataBytes = new TextEncoder().encode(additionalData);

  // extract robustness tag and ciphertext
  const robustnessTag = combinedCiphertext.slice(0, ROBUSTNESS_TAG_SIZE);
  const ciphertext = combinedCiphertext.slice(ROBUSTNESS_TAG_SIZE);

  // verify robustness tag
  const tagInput = toBase64Url(nonce) + toBase64Url(ciphertext) + additionalData;
  const tagInputBytes = new TextEncoder().encode(tagInput);

  const expectedTag = kmac256(key, tagInputBytes, {
    dkLen: ROBUSTNESS_TAG_SIZE,
    personalization: new TextEncoder().encode('RobustnessTag-v1'),
  });

  const tagMatch = equalBytes(robustnessTag, expectedTag);
  if (!tagMatch) {
    throw new Error('AEAD robustness tag mismatch');
  }

  // decrypt
  const cipher = xchacha20poly1305(key, nonce, additionalDataBytes);
  const decrypted = cipher.decrypt(ciphertext);
  return new TextDecoder().decode(decrypted);
}
