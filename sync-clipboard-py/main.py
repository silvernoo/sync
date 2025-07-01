import argparse
import asyncio
from .client import run

async def main():
    parser = argparse.ArgumentParser(description="Clipboard Sync Client")
    parser.add_argument("-a", "--address", type=str, default="127.0.0.1", help="The address of the server to connect to")
    parser.add_argument("-p", "--port", type=int, default=7878, help="The port of the server to connect to")
    parser.add_argument("-k", "--key", type=str, required=True, help="The secret key for encryption")

    args = parser.parse_args()

    try:
        await run(args.address, args.port, args.key)
    except Exception as e:
        print(f"Client error: {e}")

if __name__ == "__main__":
    asyncio.run(main())
