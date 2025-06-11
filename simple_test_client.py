# simple_test_client.py
import asyncio

HOST, PORT = '127.0.0.1', 7878

async def client(name: str, payload: bytes, is_prime: bool = False):
    reader, writer = await asyncio.open_connection(HOST, PORT)
    # discard '*' handshake
    await reader.readexactly(1)

    # build message
    msg = (b'P' + payload) if is_prime else payload
    writer.write(b'^' + msg + b'$')
    await writer.drain()

    if is_prime:
        # server replies 'prime\n' or 'composite\n'
        resp = await reader.readuntil(b'\n')
        print(f'{name}: {resp.strip().decode()}')
    else:
        data = await reader.readexactly(len(payload))
        # undo transform (+1)
        res = bytes(b - 1 for b in data)
        print(f'{name}: {res.decode()}')

    writer.close()
    await writer.wait_closed()

async def main():
    # send LongPrime first, then echo and small prime
    await asyncio.gather(
        client('LongPrime', str(2**61 - 1).encode(), True),
        client('Echo', b'echo', False),
        client('SmallPrime', b'7', True),
    )

if __name__ == '__main__':
    asyncio.run(main())
