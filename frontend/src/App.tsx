import React from 'react';
import './App.css';
import { useState, useEffect } from 'react';
import { useDebounce } from 'use-debounce';

interface Writeup {
  title: string,
  description: string,
  author: string,
  team: string,
  tags: string[],
  event: string,
  orig_writeup_link: string,
  link: string;
}

function App() {
  const [searchValue, setSearchValue] = useState<string>('');
  const [searchResults, setSearchResults] = useState<Writeup[]>();

  const [debounceSearchInput] = useDebounce(searchValue, 1000);

  const url = '';

  useEffect(() => {
    const fetchData = async () => {
      if (debounceSearchInput.trim() === '') {
        setSearchResults([]);
        return;
      }

      try {
        const response = await fetch(url + debounceSearchInput);
        const data = await response.json();
        setSearchResults(data);
      } catch (error) {
        console.error(error);
      }
    };

    fetchData();
  }, [debounceSearchInput]);

  return (
    <div className="app">
      <h2>Умный поиск по райтапам CTF</h2>
      <input id="search"
        type="search"
        placeholder="&#x1F50D; Start typing to search..."
        value={searchValue}
        onChange={(e) => setSearchValue(e.target.value)}
      />
      <div className='searchResults'>
        {searchResults?.length
          ? searchResults.map((item) =>
            <div>
              <a href={item.link}>{item.title}</a>
            </div>)
          : ''}
      </div>
    </div>
  );
}

export default App;
