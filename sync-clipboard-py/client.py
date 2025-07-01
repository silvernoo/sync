import asyncio
import pyperclip
import struct
import time
from typing import Optional

from .crypto import decrypt, encrypt, key_from_password
from .protocol import ClipboardData, serialize_clipboard_data, deserialize_clipboard_data

async def run(address: str, port: int, key_str: str):
    reader, writer = await asyncio.open_connection(address, port)
    key = key_from_password(key_str)

    print(f"Connected to server {address}:{port}")

    last_set_data: Optional[ClipboardData] = None
    last_text_sent: Optional[str] = None

    async def read_from_server():
        nonlocal last_set_data
        nonlocal last_text_sent
        while True:
            try:
                len_bytes = await reader.readexactly(4)
                msg_len = struct.unpack(">I", len_bytes)[0]
                encrypted_data = await reader.readexactly(msg_len)

                decrypted_data = decrypt(encrypted_data, key)
                data = deserialize_clipboard_data(decrypted_data)

                print(f"Received data from server: {data}")

                last_set_data = data # Store the received data

                if data.type == "text":
                    pyperclip.copy(data.value)
                    last_text_sent = data.value
                elif data.type == "image":
                    print("Image data received, but image clipboard is not supported yet.")

            except asyncio.IncompleteReadError:
                print("Connection closed by server.")
                break
            except Exception as e:
                print(f"Error reading from server: {e}")
                break

    async def write_to_server():
        nonlocal last_set_data
        nonlocal last_text_sent
        while True:
            await asyncio.sleep(0.5)

            current_text = pyperclip.paste()
            current_data: Optional[ClipboardData] = None

            if current_text and current_text != last_text_sent:
                current_data = ClipboardData.text(current_text)

            if current_data:
                should_send = True
                if last_set_data and last_set_data.type == current_data.type:
                    if last_set_data.type == "text" and last_set_data.value == current_data.value:
                        should_send = False
                    # For images, a more robust comparison would be needed.
                    # For now, we'll assume if it was just set, we don't resend.
                    elif last_set_data.type == "image":
                        should_send = False

                if should_send:
                    print(f"Detected new clipboard data: {current_data}")
                    serialized_data = serialize_clipboard_data(current_data)
                    encrypted_data = encrypt(serialized_data, key)

                    try:
                        writer.write(struct.pack(">I", len(encrypted_data)))
                        writer.write(encrypted_data)
                        await writer.drain()
                        last_text_sent = current_text
                    except Exception as e:
                        print(f"Error writing to server: {e}")
                        break

                last_set_data = None # Clear last_set_data after checking

    await asyncio.gather(read_from_server(), write_to_server())

    writer.close()
    await writer.wait_closed()
