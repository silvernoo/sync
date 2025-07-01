import hashlib
from cryptography.hazmat.primitives.ciphers import Cipher, algorithms, modes
from cryptography.hazmat.backends import default_backend
from cryptography.hazmat.primitives import hashes
import os

def key_from_password(password: str) -> bytes:
    hasher = hashlib.sha256()
    hasher.update(password.encode('utf-8'))
    return hasher.digest()

def encrypt(data: bytes, key: bytes) -> bytes:
    cipher = Cipher(algorithms.AES(key), modes.GCM(os.urandom(12)), backend=default_backend())
    encryptor = cipher.encryptor()
    nonce = encryptor.nonce
    ciphertext = encryptor.update(data) + encryptor.finalize()
    return nonce + ciphertext + encryptor.tag

def decrypt(data: bytes, key: bytes) -> bytes:
    nonce = data[:12]
    ciphertext = data[12:-16] # Last 16 bytes are the tag
    tag = data[-16:]

    cipher = Cipher(algorithms.AES(key), modes.GCM(nonce, tag), backend=default_backend())
    decryptor = cipher.decryptor()
    return decryptor.update(ciphertext) + decryptor.finalize()
