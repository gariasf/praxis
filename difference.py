"""Difference image like tests/goldens.rs: the after image dimmed to a third,
every pixel whose largest channel delta exceeds the tolerance in opaque red.
Usage: golden_diff.py before.png after.png out.png [tolerance]"""
import struct, sys, zlib

def read_png(path):
    data = open(path, 'rb').read()
    pos, idat = 8, b''
    while pos < len(data):
        length = struct.unpack('>I', data[pos:pos + 4])[0]
        kind, chunk = data[pos + 4:pos + 8], data[pos + 8:pos + 8 + length]
        pos += 12 + length
        if kind == b'IHDR':
            width, height, _, color_type = struct.unpack('>IIBB', chunk[:10])
        elif kind == b'IDAT':
            idat += chunk
    channels = {6: 4, 2: 3}[color_type]
    raw, stride = zlib.decompress(idat), width * channels
    pixels, previous, offset = bytearray(), bytearray(stride), 0
    for _ in range(height):
        kind = raw[offset]; offset += 1
        line = bytearray(raw[offset:offset + stride]); offset += stride
        for x in range(stride):
            left = line[x - channels] if x >= channels else 0
            up = previous[x]
            up_left = previous[x - channels] if x >= channels else 0
            if kind == 1: line[x] = (line[x] + left) & 255
            elif kind == 2: line[x] = (line[x] + up) & 255
            elif kind == 3: line[x] = (line[x] + ((left + up) >> 1)) & 255
            elif kind == 4:
                estimate = left + up - up_left
                distances = (abs(estimate - left), abs(estimate - up), abs(estimate - up_left))
                predictor = left if distances[0] <= distances[1] and distances[0] <= distances[2] else (up if distances[1] <= distances[2] else up_left)
                line[x] = (line[x] + predictor) & 255
        pixels += line; previous = line
    rgb = bytearray()
    for index in range(0, len(pixels), channels):
        rgb += pixels[index:index + 3]
    return width, height, rgb

def write_png(path, width, height, rgb):
    rows = b''.join(b'\x00' + bytes(rgb[y * width * 3:(y + 1) * width * 3]) for y in range(height))
    chunk = lambda kind, body: struct.pack('>I', len(body)) + kind + body + struct.pack('>I', zlib.crc32(kind + body) & 0xffffffff)
    header = struct.pack('>IIBBBBB', width, height, 8, 2, 0, 0, 0)
    open(path, 'wb').write(b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', header) + chunk(b'IDAT', zlib.compress(rows, 9)) + chunk(b'IEND', b''))

tolerance = int(sys.argv[4]) if len(sys.argv) > 4 else 1
width, height, before = read_png(sys.argv[1])
other_width, other_height, after = read_png(sys.argv[2])
assert (width, height) == (other_width, other_height)
out, changed, beyond, largest = bytearray(), 0, 0, 0
for index in range(0, len(after), 3):
    delta = max(abs(before[index + k] - after[index + k]) for k in range(3))
    changed += delta > 0; largest = max(largest, delta)
    if delta > tolerance:
        beyond += 1; out += bytes((255, 0, 0))
    else:
        out += bytes(value // 3 for value in after[index:index + 3])
write_png(sys.argv[3], width, height, out)
print(f'{sys.argv[2]}: {changed} pixels changed, {beyond} beyond {tolerance}, max delta {largest}')
