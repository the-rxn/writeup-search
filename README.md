# Умный поиск по райтапам CTF

**Mentor:** @yorxx

## Описание

Стоит признать, что многие задания на CTF-соревнованиях являются производными от других заданий. Поэтому эффективным может быть поиск по райтапам с предыдущих соревнований. Многие райтапы собраны на CTFTime, но поиск по ним довольно неудобный.
CTFTime имеет свой API (https://ctftime.org/api/).

## Что мы хотим

1. Проведите парсинг райтапов с CTFTime и\или другого согласованного с ментором источника.
2. Примените и обоснованнуйте метод обработки данных для реализации семантического поиска по данным.
3. Предоставьте интерфейс для поиска.
   
## Критерии оценивания

Корректность работы, удобство использования, обоснованность применяемых методов и инструментов.

## Как запустить
Перед началом использования/разработки, прописываем: 
`pip install -r requirements.txt` 

### Как скачать райтапы
В корне проекта пишем 
```
mkdir writeups
python scripts/download_writeups.py
```
Скачиваться будет долго, к сожаланию нормально распаралерить не получилось потому что ctftime не выдерживает большое количество одновременных запросов.


