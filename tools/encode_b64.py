import base64

def encode_binary_string(binary_string):
    # Convert the binary string to bytes
    binary_bytes = int(binary_string, 2).to_bytes((len(binary_string) + 7) // 8, byteorder='big')

    # Encode the bytes using base64
    base64_encoded = base64.b64encode(binary_bytes)

    # Convert the base64 bytes to a string
    return base64_encoded.decode()

if __name__ == '__main__':
    while True:
        inp = input("Enter binary string: ")
        print(encode_binary_string(inp))