# @inkrypt/crypto

Quantum-safe primitives for apps: authenticated encryption, hashing, random IDs, and post-quantum signatures. Built on `@noble/*` libraries.

## Usage

```ts
import {
  // AEAD
  encryptMessage,
  decryptMessage,
  // Hash
  hash,
  // Random
  getRandomBytes,
  generateId,
  // Signatures (PQC)
  generatePqcSignatureKeyPair,
  sign,
  verifySignature,
} from "@inkrypt/crypto";

// AEAD: XChaCha20-Poly1305 + KMAC-256 robustness tag
const { ciphertext, key, nonce } = encryptMessage("hello", "ctx");
const plaintext = decryptMessage(ciphertext, "ctx", key, nonce);

// Hash: BLAKE3
const digest = hash("data"); // Uint8Array

// Random
const bytes = getRandomBytes(32);
const id = generateId(); // url-safe base64 id with higher entropy than uuidv4 or uuidv7

// Post-quantum signatures (ML-DSA-87)
const { publicKey, secretKey } = generatePqcSignatureKeyPair();
const content = { id: "123", amount: 10 };
const sig = sign(content, "MyApp:v1", secretKey);
const ok = verifySignature(content, "MyApp:v1", sig, publicKey);
```

## Notes

- AEAD uses 32-byte keys and 24-byte nonces (XChaCha20-Poly1305) plus a KMAC-256 robustness tag over `nonce`+`ciphertext`+`AAD`.
- `sign`/`verifySignature` canonicalize JSON and prepend a domain context string to prevent cross-domain replay.
- Returns `Uint8Array` for binary values, encode as needed.
- Requires modern browsers or Node.js with Web Crypto.

## Built With

- [@noble/ciphers](https://github.com/paulmillr/ciphers) - Modern cryptographic ciphers
- [@noble/hashes](https://github.com/paulmillr/hashes) - Fast cryptographic hash functions
- [@noble/post-quantum](https://github.com/paulmillr/post-quantum) - Post-quantum cryptography
- [canonicalize](https://github.com/erdtman/canonicalize) - Crypto-safe and deterministic JSON serialization
