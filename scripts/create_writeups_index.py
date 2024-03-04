import marqo

mq = marqo.Client(url='http://localhost:8882')

mq.create_index("writeups")
print("created index!")
