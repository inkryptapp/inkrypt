import { randomBytes } from '@noble/hashes/utils';
import { toBase64Url } from './base64';

export const ID_BYTES_LENGTH = 24; // (bytes / 3) * 4 = 32 chars

/**
 * Cryptographically secure PRNG. Uses internal OS-level `crypto.getRandomValues`.
 */
export function getRandomBytes(bytesLength = 32): Uint8Array {
  return randomBytes(bytesLength);
}

/**
 * Generates a URL‑safe, fixed‑length random ID.
 *
 * - UUID‑like length but higher entropy than `uuidv4` or `uuidv7`
 *
 * @param bytesLength Default 24 bytes (192 bits) → 32 characters
 */
export function generateId(bytesLength = ID_BYTES_LENGTH): string {
  return toBase64Url(getRandomBytes(bytesLength));
}
