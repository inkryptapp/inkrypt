import { describe, expect, test } from 'vitest';
import { decryptMessage, encryptMessage } from './aead';
import { getRandomBytes } from './random';

const KEY_LENGTH = 32; // bytes
const NONCE_LENGTH = 24; // bytes

describe('encryptMessage', () => {
  test('should encrypt a string message and return expected structure', () => {
    const message = 'Hello, World!';
    const additionalData = 'test-data';

    const result = encryptMessage(message, additionalData);

    expect(result).toHaveProperty('ciphertext');
    expect(result).toHaveProperty('key');
    expect(result).toHaveProperty('nonce');
    expect(result.ciphertext).toBeInstanceOf(Uint8Array);
    expect(result.key).toBeInstanceOf(Uint8Array);
    expect(result.nonce).toBeInstanceOf(Uint8Array);
    expect(result.key.length).toBe(KEY_LENGTH);
    expect(result.nonce.length).toBe(NONCE_LENGTH);
    expect(result.ciphertext.length).toBeGreaterThan(0);
  });

  test('should encrypt a Uint8Array message', () => {
    const message = new TextEncoder().encode('Hello, World!');
    const additionalData = 'test-data';

    const result = encryptMessage(message, additionalData);

    expect(result.ciphertext).toBeInstanceOf(Uint8Array);
    expect(result.ciphertext.length).toBeGreaterThan(0);
  });

  test('should use provided key and nonce when given', () => {
    const message = 'Hello, World!';
    const additionalData = 'test-data';
    const customKey = getRandomBytes(KEY_LENGTH);
    const customNonce = getRandomBytes(NONCE_LENGTH);

    const result = encryptMessage(message, additionalData, customKey, customNonce);

    expect(result.key).toEqual(customKey);
    expect(result.nonce).toEqual(customNonce);
  });

  test('should generate different ciphertexts for same message with random keys', () => {
    const message = 'Hello, World!';
    const additionalData = 'test-data';

    const result1 = encryptMessage(message, additionalData);
    const result2 = encryptMessage(message, additionalData);

    expect(result1.ciphertext).not.toEqual(result2.ciphertext);
    expect(result1.key).not.toEqual(result2.key);
    expect(result1.nonce).not.toEqual(result2.nonce);
  });

  test('should throw error for invalid key size', () => {
    const message = 'Hello, World!';
    const additionalData = 'test-data';
    const invalidKey = getRandomBytes(16);

    expect(() => {
      encryptMessage(message, additionalData, invalidKey);
    }).toThrow(`${KEY_LENGTH}-byte key is required for XChaCha20-Poly1305`);
  });

  test('should include robustness tag in ciphertext', () => {
    const message = 'Hello, World!';
    const additionalData = 'test-data';

    const result = encryptMessage(message, additionalData);

    expect(result.ciphertext.length).toBeGreaterThan(KEY_LENGTH);
  });

  test('should handle empty message', () => {
    const message = '';
    const additionalData = 'test-data';

    const result = encryptMessage(message, additionalData);

    expect(result.ciphertext.length).toBeGreaterThan(KEY_LENGTH);
  });
});

describe('decryptMessage', () => {
  test('should decrypt a valid ciphertext', () => {
    const message = 'Hello, World!';
    const additionalData = 'test-data';
    const key = getRandomBytes(KEY_LENGTH);
    const nonce = getRandomBytes(NONCE_LENGTH);

    const encrypted = encryptMessage(message, additionalData, key, nonce);
    const decrypted = decryptMessage(encrypted.ciphertext, additionalData, key, nonce);

    expect(decrypted).toBe(message);
  });

  test('should throw error for invalid robustness tag', () => {
    const message = 'Hello, World!';
    const additionalData = 'test-data';
    const key = getRandomBytes(KEY_LENGTH);
    const nonce = getRandomBytes(NONCE_LENGTH);

    const encrypted = encryptMessage(message, additionalData, key, nonce);

    // Corrupt the robustness tag (first 32 bytes)
    const corruptedCiphertext = new Uint8Array(encrypted.ciphertext);
    corruptedCiphertext[0] ^= 1;

    expect(() => {
      decryptMessage(corruptedCiphertext, additionalData, key, nonce);
    }).toThrow('AEAD robustness tag mismatch');
  });

  test('should throw error for wrong key', () => {
    const message = 'Hello, World!';
    const additionalData = 'test-data';
    const key = getRandomBytes(KEY_LENGTH);
    const wrongKey = getRandomBytes(KEY_LENGTH);
    const nonce = getRandomBytes(NONCE_LENGTH);

    const encrypted = encryptMessage(message, additionalData, key, nonce);

    expect(() => {
      decryptMessage(encrypted.ciphertext, additionalData, wrongKey, nonce);
    }).toThrow('AEAD robustness tag mismatch');
  });

  test('should throw error for wrong nonce', () => {
    const message = 'Hello, World!';
    const additionalData = 'test-data';
    const key = getRandomBytes(KEY_LENGTH);
    const nonce = getRandomBytes(NONCE_LENGTH);
    const wrongNonce = getRandomBytes(NONCE_LENGTH);

    const encrypted = encryptMessage(message, additionalData, key, nonce);

    expect(() => {
      decryptMessage(encrypted.ciphertext, additionalData, key, wrongNonce);
    }).toThrow('AEAD robustness tag mismatch');
  });

  test('should throw error for wrong additional data', () => {
    const message = 'Hello, World!';
    const additionalData = 'test-data';
    const wrongAdditionalData = 'wrong-data';
    const key = getRandomBytes(KEY_LENGTH);
    const nonce = getRandomBytes(NONCE_LENGTH);

    const encrypted = encryptMessage(message, additionalData, key, nonce);

    expect(() => {
      decryptMessage(encrypted.ciphertext, wrongAdditionalData, key, nonce);
    }).toThrow('AEAD robustness tag mismatch');
  });

  test('should handle empty message decryption', () => {
    const message = '';
    const additionalData = 'test-data';
    const key = getRandomBytes(KEY_LENGTH);
    const nonce = getRandomBytes(NONCE_LENGTH);

    const encrypted = encryptMessage(message, additionalData, key, nonce);
    const decrypted = decryptMessage(encrypted.ciphertext, additionalData, key, nonce);

    expect(decrypted).toBe('');
  });
});

describe('encrypt/decrypt integration', () => {
  test('should handle various message types and sizes', () => {
    const testCases = [
      'Hello, World!',
      'Lorem ipsum dolor sit amet, consectetur adipiscing elit.',
      '🎉 Unicode and emojis 🚀',
      'A'.repeat(1000), // Long message
      JSON.stringify({ data: 'complex object', nested: { value: 42 } }),
    ];

    testCases.forEach((message) => {
      const additionalData = 'integration-test';
      const encrypted = encryptMessage(message, additionalData);
      const decrypted = decryptMessage(
        encrypted.ciphertext,
        additionalData,
        encrypted.key,
        encrypted.nonce,
      );

      expect(decrypted).toBe(message);
    });
  });

  test('should handle various additional data', () => {
    const message = 'Test message';
    const additionalDataCases = [
      '',
      'simple',
      'with spaces and symbols !@#$%^&*()',
      '🔒 Unicode in additional data',
      JSON.stringify({ metadata: 'test', version: 1 }),
    ];

    additionalDataCases.forEach((additionalData) => {
      const encrypted = encryptMessage(message, additionalData);
      const decrypted = decryptMessage(
        encrypted.ciphertext,
        additionalData,
        encrypted.key,
        encrypted.nonce,
      );

      expect(decrypted).toBe(message);
    });
  });

  test('should maintain data integrity over multiple encrypt/decrypt cycles', () => {
    let message = 'Initial message';
    const additionalData = 'cycle-test';

    for (let i = 0; i < 5; i++) {
      const encrypted = encryptMessage(message, additionalData);
      const decrypted = decryptMessage(
        encrypted.ciphertext,
        additionalData,
        encrypted.key,
        encrypted.nonce,
      );

      expect(decrypted).toBe(message);
      message = `Cycle ${i + 1}: ${decrypted}`;
    }
  });

  test('should produce different ciphertexts for same plaintext with different additional data', () => {
    const message = 'Same message';
    const additionalData1 = 'context1';
    const additionalData2 = 'context2';
    const key = getRandomBytes(KEY_LENGTH);
    const nonce = getRandomBytes(NONCE_LENGTH);

    const encrypted1 = encryptMessage(message, additionalData1, key, nonce);
    const encrypted2 = encryptMessage(message, additionalData2, key, nonce);

    expect(encrypted1.ciphertext).not.toEqual(encrypted2.ciphertext);

    // Should decrypt correctly with matching additional data
    expect(decryptMessage(encrypted1.ciphertext, additionalData1, key, nonce)).toBe(
      message,
    );
    expect(decryptMessage(encrypted2.ciphertext, additionalData2, key, nonce)).toBe(
      message,
    );
  });
});
