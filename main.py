import marqo
import json
import pprint

MAX_BATCH_SIZE=128

def read_writeups_json(filepath: str):
    with open(filepath, "r") as file:
        return json.load(file)

# mq = marqo.Client(url='http://localhost:8882')
mq = marqo.Client(url='http://184.148.225.95:41356')
INDEX_NAME = "writeups"
BATCH_SIZE=4
STOPPED_AT=384

# try:
#     mq.delete_index(INDEX_NAME)
# except Exception as e:
#     print(e)
mq.create_index(INDEX_NAME)
documents = read_writeups_json("./writeups_full_tags_orig_clean.json")
n = BATCH_SIZE
documents_batched= [documents[i * n:(i + 1) * n] for i in range((len(documents) + n - 1) // n )]
# print(documents_batched[0][0])
num = 0
for i in range(0, len(documents_batched)):
# for i in range(STOPPED_AT/BATCH_SIZE, len(documents_batched)):
    res = mq.index(INDEX_NAME).add_documents(documents_batched[i], tensor_fields=['title', 'description', 'tags'])
    # res = mq.index(INDEX_NAME).add_documents(documents_batched[i], tensor_fields=['title', 'description', 'tags'], device="cuda")
    num += BATCH_SIZE
    print(res)
    print(f"Loaded {num} writeups into embedding")

results = mq.index(INDEX_NAME).search(
    q="What to do if variable names are jumbled (java)?", limit=1
)
pprint.pprint(results)
