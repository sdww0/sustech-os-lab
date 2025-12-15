# Reference: https://github.com/asterinas/asterinas/issues/964

import re

def process_stack_trace(file_path):
    with open(file_path, 'r') as file:
        lines = file.readlines()

    captures = []
    current_capture = []

    for line in lines:
        if not line.startswith('#'):
            continue
        if line.startswith('#0'):
            if current_capture:
                captures.append(current_capture)
                current_capture = []
        processed = line.strip()
        # remove all things between < and >, use bracket matching
        cut_generic = ''
        cnt = 0
        for c in processed:
            if c == '<':
                cnt += 1
            if cnt == 0:
                cut_generic += c
            if c == '>':
                cnt -= 1
        processed = cut_generic
        # remove all things like "::{impl#70}"
        processed = re.sub(r'::\{.*?\}', '', processed)
        # remove all "(...)"
        processed = processed.replace('(...)', '')
        # remove all "()"
        processed = processed.replace('()', '')
        # remove all things like "0xffffffff8819d0fb in"
        processed = re.sub(r'0x[0-9a-f]+ in', '', processed)

        # split by spaces, the first is number and the second is function name
        parts = [s for s in processed.split(' ') if s != '']

        current_capture.append(parts[1])

    if current_capture:
        captures.append(current_capture)

    folded = {} # { bt: value }

    for capture in captures:
        bt_from_butt = []
        for frame in reversed(capture):
            bt_from_butt.append(frame)
        folded_key = ';'.join(bt_from_butt)
        if folded_key in folded:
            folded[folded_key] += 1
        else:
            folded[folded_key] = 1

    with open('out.folded', 'w') as out_file:
        for key, v in folded.items():
            out_file.write(f"{key} {v}\n")

if __name__ == "__main__":
    process_stack_trace('gdb_perf.log')
