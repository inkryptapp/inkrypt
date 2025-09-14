import { blake3 } from '@noble/hashes/blake3';

/**
 * Computes a BLAKE3 hash of the input.
 *
 * @param message Input message as string or `Uint8Array`.
 * @param outputLen Desired output length in bytes (default 64).
 * @returns Hash digest as `Uint8Array` of length `outputLen`.
 */
export function hash(message: string | Uint8Array, outputLen = 64): Uint8Array {
  return blake3(
    typeof message === 'string' ? new TextEncoder().encode(message) : message,
    { dkLen: outputLen },
  );
}
