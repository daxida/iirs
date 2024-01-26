def test_equality(cpp_folder, rust_folder):
	with open(f"{cpp_folder}/IUPACpal.out") as f:
		expected = f.read().strip()

	with open(f"{rust_folder}/IUPACpalrs.out") as f:
		received = f.read().strip()

	expected = expected.replace("Palindromes:\n", "")
	received = received.replace("Palindromes:\n", "")
	expected_lines = expected.split("\n\n")[1:] # remove header
	received_lines = received.split("\n\n")[1:]

	def block_sort(block: str):
		if len(block.strip()) == 0: # empty lines
			return (1e9,)
		fst, scd, thd = block.split("\n")
		a, b, c = fst.split()
		d, e, f = thd.split()
		return (int(a), int(c), int(d), int(f))

	expected_lines.sort(key=block_sort)
	received_lines.sort(key=block_sort)

	# Line by line
	for el, rl in zip(expected_lines, received_lines):
		assert el == rl, f"Received line:\n{rl}\nbut expected:\n{el}"

	assert len(expected_lines) == len(received_lines)
	print("Ok")
	print(f"Compared {len(expected_lines) - 1} Palindromes")

# test_equality()