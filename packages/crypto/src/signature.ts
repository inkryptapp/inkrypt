import { ml_dsa87 } from '@noble/post-quantum/ml-dsa';
import canonicalize from 'canonicalize';

/**
 * Generates a post‑quantum signature keypair using ML‑DSA‑87.
 *
 * @returns Object with `secretKey` and `publicKey` as `Uint8Array`s.
 */
export function generatePqcSignatureKeyPair(): {
  secretKey: Uint8Array;
  publicKey: Uint8Array;
} {
  return ml_dsa87.keygen();
}

/**
 * Signs canonicalized content with ML‑DSA‑87, bound to a domain context.
 *
 * - Content is canonicalized (deterministic JSON) before signing.
 * - `signatureContext` provides domain separation and must match at verify time.
 *
 * @param content JSON object with string/number values to be canonicalized and signed.
 * @param signatureContext Context string mixed into the message prior to signing.
 * @param privateKey Secret key (`Uint8Array`) produced by `generatePqcSignatureKeyPair`.
 * @returns Signature bytes as `Uint8Array`.
 * @throws Error if `content` cannot be canonicalized into a string.
 */
export function sign(
  content: Record<string, string | number>,
  signatureContext: string,
  privateKey: Uint8Array,
): Uint8Array {
  const serializedContent = canonicalize(content);
  if (typeof serializedContent !== 'string') {
    throw new Error('Invalid content');
  }
  const message = new TextEncoder().encode(signatureContext + serializedContent);
  return ml_dsa87.sign(message, privateKey);
}

/**
 * Verifies an ML‑DSA‑87 signature over canonicalized content and signature context.
 *
 * @param content JSON object with string/number values originally signed.
 * @param signatureContext Context string that must match what the signer used.
 * @param signature Signature bytes to verify.
 * @param publicKey Public key corresponding to the signer.
 * @returns `true` if the signature is valid; otherwise `false`.
 */
export function verifySignature(
  content: Record<string, string | number>,
  signatureContext: string,
  signature: Uint8Array,
  publicKey: Uint8Array,
): boolean {
  const serializedContent = canonicalize(content);
  if (typeof serializedContent !== 'string') {
    return false;
  }
  const message = new TextEncoder().encode(signatureContext + serializedContent);
  return ml_dsa87.verify(signature, message, publicKey);
}
