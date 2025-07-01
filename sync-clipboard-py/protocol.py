import struct

class ClipboardData:
    def __init__(self, type, value):
        self.type = type
        self.value = value

    @staticmethod
    def text(text_content: str):
        return ClipboardData("text", text_content)

    @staticmethod
    def image(width: int, height: int, bytes_content: bytes):
        return ClipboardData("image", {"width": width, "height": height, "bytes": bytes_content})

    def __repr__(self):
        if self.type == "text":
            return f"ClipboardData.text(text_content='{self.value}')"
        elif self.type == "image":
            return f"ClipboardData.image(width={self.value['width']}, height={self.value['height']}, bytes=<bytes len={len(self.value['bytes'])}>)"
        return f"ClipboardData(type='{self.type}', value='{self.value}')"

    def __eq__(self, other):
        if not isinstance(other, ClipboardData):
            return NotImplemented
        return self.type == other.type and self.value == other.value

def serialize_clipboard_data(data: ClipboardData) -> bytes:
    if data.type == "text":
        # Variant tag for Text (0) + length of string + string bytes
        text_bytes = data.value.encode('utf-8')
        return struct.pack("<I", 0) + struct.pack("<Q", len(text_bytes)) + text_bytes
    elif data.type == "image":
        # Variant tag for Image (1) + width + height + length of bytes + image bytes
        image_bytes = data.value["bytes"]
        return struct.pack("<I", 1) + \
               struct.pack("<I", data.value["width"]) + \
               struct.pack("<I", data.value["height"]) + \
               struct.pack("<Q", len(image_bytes)) + \
               image_bytes
    else:
        raise ValueError("Unsupported ClipboardData type")

def deserialize_clipboard_data(data_bytes: bytes) -> ClipboardData:
    # Read variant tag
    variant_tag = struct.unpack("<I", data_bytes[0:4])[0]
    offset = 4

    if variant_tag == 0:  # Text
        text_len = struct.unpack("<Q", data_bytes[offset:offset+8])[0]
        offset += 8
        text_content = data_bytes[offset:offset+text_len].decode('utf-8')
        return ClipboardData.text(text_content)
    elif variant_tag == 1:  # Image
        width = struct.unpack("<I", data_bytes[offset:offset+4])[0]
        offset += 4
        height = struct.unpack("<I", data_bytes[offset:offset+4])[0]
        offset += 4
        image_len = struct.unpack("<Q", data_bytes[offset:offset+8])[0]
        offset += 8
        image_bytes = data_bytes[offset:offset+image_len]
        return ClipboardData.image(width, height, image_bytes)
    else:
        raise ValueError("Unknown ClipboardData variant tag")