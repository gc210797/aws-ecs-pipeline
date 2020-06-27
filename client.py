import argparse
import asyncio
import signal
import sys

import aiohttp

queue = asyncio.Queue()


async def start_client(url, loop):
    ws = await aiohttp.ClientSession().ws_connect(url, autoping=False,
                                                  autoclose=False)

    def stdin_callback():
        line = sys.stdin.buffer.readline().decode('utf-8')
        if not line:
            loop.stop()
        else:
            asyncio.ensure_future(queue.put(ws.send_str(line)))

    loop.add_reader(sys.stdin, stdin_callback)

    close = False
    close_lock = asyncio.Lock()

    async def dispatch():
        while True:
            msg = await ws.receive()
            if msg.type == aiohttp.WSMsgType.TEXT:
                print('Text:', msg.data.strip())
            elif msg.type == aiohttp.WSMsgType.BINARY:
                print('Binary:', msg.data)
            elif msg.type == aiohttp.WSMsgType.PING:
                await ws.pong()
            elif msg.type == aiohttp.WSMsgType.PONG:
                print('pong received')
            else:
                async with close_lock:
                    close = True
                if msg.type == aiohttp.WSMsgType.CLOSE:
                    await ws.close()
                elif msg.type == aiohttp.WSMsgType.ERROR:
                    print('Error during recieve %s' % ws.exception())
                elif msg.type == aiohttp.WSMsgType.CLOSED:
                    pass
                break

    async def send_ping():
        while True:
            async with close_lock:
                if close is True:
                    break
            await asyncio.sleep(4)
            await ws.ping("1")

    await asyncio.wait([dispatch(), send_ping()])


async def tick():
    while True:
        await (await queue.get())


async def main(url, loop):
    await asyncio.wait([start_client(url, loop), tick()])

ARGS = argparse.ArgumentParser(
    description="websocket console client for wssrv.py example.")
ARGS.add_argument("--host", action="store", dest="host",
                  default='127.0.0.1', help='host name')
ARGS.add_argument("--port", action="store", dest='port',
                  default=8080, type=int, help="port number")

if __name__ == "__main__":
    args = ARGS.parse_args()

    url = "http://{}:{}/ws/".format(args.host, args.port)

    loop = asyncio.get_event_loop()
    loop.add_signal_handler(signal.SIGINT, loop.stop)
    loop.run_until_complete(main(url, loop))
