import { describe, expect, test } from 'vitest';
import { generateId, getRandomBytes, ID_BYTES_LENGTH } from './random';

const bytesToLen = (bytes: number) => (bytes / 3) * 4;

describe('getRandomBytes', () => {
  test('should return Uint8Array of specified length', () => {
    const result = getRandomBytes(32);
    expect(result).toBeInstanceOf(Uint8Array);
    expect(result.length).toBe(32);
  });

  test('should return different values on subsequent calls', () => {
    const result1 = getRandomBytes(32);
    const result2 = getRandomBytes(32);
    expect(result1).not.toEqual(result2);
  });

  test('should handle zero length', () => {
    const result = getRandomBytes(0);
    expect(result).toBeInstanceOf(Uint8Array);
    expect(result.length).toBe(0);
  });

  test('should handle various lengths', () => {
    [1, 8, 16, 32, 64, 128].forEach((length) => {
      const result = getRandomBytes(length);
      expect(result.length).toBe(length);
      expect(result).toBeInstanceOf(Uint8Array);
    });
  });
});

describe('generateId', () => {
  test('should return a string', () => {
    const result = generateId();
    expect(typeof result).toBe('string');
  });

  test('should return different values on subsequent calls', () => {
    const result1 = generateId();
    const result2 = generateId();
    expect(result1).not.toBe(result2);
  });

  test('should use default length of 24 bytes (32 chars base64)', () => {
    const result = generateId();
    expect(result.length).toBe(bytesToLen(ID_BYTES_LENGTH));
  });

  test('should accept custom byte length', () => {
    const r1Bytes = 12;
    const r2Bytes = 18;
    const result1 = generateId(r1Bytes);
    const result2 = generateId(r2Bytes);
    expect(result1.length).toBe(bytesToLen(r1Bytes));
    expect(result2.length).toBe(bytesToLen(r2Bytes));
  });

  test('should generate valid base64url strings', () => {
    const result = generateId();
    const base64urlRegex = /^[A-Za-z0-9_-]*$/;
    expect(base64urlRegex.test(result)).toBe(true);
  });

  test('should handle zero length', () => {
    const result = generateId(0);
    expect(result).toBe('');
  });
});
