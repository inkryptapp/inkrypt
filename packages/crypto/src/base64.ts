import { fromUint8Array, toUint8Array } from 'js-base64';

/**
 * Encodes bytes into a URL-safe Base64 string (without padding).
 *
 * @param u8a Input bytes to encode.
 * @returns URL-safe Base64 string.
 */
export function toBase64Url(u8a: Uint8Array): string {
  // TODO: switch to Uint8Array.toBase64:
  // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Uint8Array/toBase64
  return fromUint8Array(u8a, true);
}

/**
 * Decodes a Base64 or URL-safe Base64 string into bytes. Padding is optional.
 *
 * @param b64 Base64 input string.
 * @returns Decoded bytes as `Uint8Array`.
 */
export function fromBase64(b64: string): Uint8Array {
  // TODO: switch to Uint8Array.fromBase64:
  // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Uint8Array/fromBase64
  return toUint8Array(b64);
}
