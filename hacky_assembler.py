import sys

import argh


# {name: opcode (base value without args)}
OPS = {
	'bank': 0, # special, other values added later
	'immd': 0x00,
	'bit': 0x20,
	'onto': 0x30,
	'mix': 0x14,
	'inc': 0x18,
	'dec': 0x1c,
	'load': 0x10,
	'invalid11': 0x11,
	'invalid12': 0x12,
	'halt': 0x13,
}


def parse_int(value, nibble=False):
	value = int(value, 10)
	if nibble:
		# allow strictly 0-15
		if not (0 <= value < 16):
			raise ValueError("Value out of range")
	else:
		# allow signed or unsigned, translate signed to two's complement form
		if not (-127 <= value < 256):
			raise ValueError("Value out of range")
		value %= 256
	return value


def parse_reg(value):
	return {
		'a': 0,
		'ip': 1,
		'p': 2,
		'[p]': 3,
	}[value]


def main(filename):
	with open(filename) as f:
		lines = f.read().split('\n')
	# remove comments and whitespace, coerce case
	lines = [line.split('#', 1)[0].strip().lower() for line in lines]
	# remove empty lines
	lines = [line for line in lines if line]

	banks = {} # {bank: [data]}
	bank = 0
	ip = 0
	for line in lines:
		# check for [BANK:]ADDR: directive
		if ':' in line:
			parts = map(parse_int, line.split(':'))
			if len(parts) == 2:
				ip = parts[0]
			else:
				bank, ip = parts[:1]
			continue
		args = line.split()
		instr, args = args[0], args[1:]
		# check for data directive
		if instr == 'data':
			arg, = args
			value = parse_int(arg)
		else:
			value = 0
			# check for leading +/- (none is the same as -)
			if instr.startswith('+'):
				value += 0x80
				instr = instr[1:]
			elif instr.startswith('-'):
				instr = instr[1:]
			# check for leading ! to indicate setting cond
			if instr.startswith('!'):
				value += 0x40
				instr = instr[1:]
			# add opcode base
			value += OPS[instr]
			# add args
			if instr == 'bank':
				arg, = args
				value += {
					'ip': 0x3f,
					'p': 0x2f,
				}[arg]
			elif instr == 'immd':
				arg, = args
				value += parse_int(arg, nibble=True)
			elif instr in ('bit', 'onto'):
				src, dest = map(parse_reg, args)
				value += (dest << 2) + src
			elif instr in ('mix', 'inc', 'dec'):
				arg, = args
				value += parse_reg(arg)
			else:
				if args:
					raise ValueError("No args expected")
		banks.setdefault(bank, [0] * 256)[ip] = value
		ip += 1
		if ip >= 256:
			raise Exception("Bank overflow")

	# Collect all defined banks + gaps
	result = []
	for bank in range(max(banks) + 1):
		result += banks.get(bank, [0] * 256)
	# Convert to bytestring (thankfully easy in py2)
	result = ''.join(chr(v) for v in result)
	# Write to stdout
	sys.stdout.write(result)


if __name__ == '__main__':
	argh.dispatch_command(main)
