import { useState, useEffect, useMemo } from 'react';
import { useDebounce } from 'use-debounce';
import Markdown from 'react-markdown';
import { dracula } from "react-syntax-highlighter/dist/esm/styles/hljs";
import SyntaxHighlighter from "react-syntax-highlighter";
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

  useEffect(() => {
    const fetchData = async () => {
      if (debounceSearchInput.trim() === '') {
        setSearchResults([]);
        return;
      }

      try {
        let response = await fetch("https://webapp.nixrs.ru/indexes/writeups/search", {
          method: 'POST',
          headers: {
            'Content-type': 'application/json',
          },
          body: JSON.stringify({
            q: searchValue,
            limit: 10,
          })
        });
        const data = (await response.json()).hits as Writeup[];
        console.log(data);
        setSearchResults(data);
      } catch (error) {
        console.error(error);
      }
    };

    fetchData();
  }, [debounceSearchInput]);
  const rendered_results = useMemo(() => {
    return         searchResults?.length
          ? searchResults.map((item) =>
            <div className="collapse collapse-plus bg-neutral my-4">
              <div className="collapse-title text-xl font-medium">
                {item.title == '' ? 'No TITLE' : item.title}
              </div>
              <input type="checkbox" />
              <div className="collapse-content text-left prose mx-2">
                <h2 className="text-accent">Author: {item.author}</h2>
                <h2 className="text-secondary"><a className="text-secondary" href={item.link}>Link to source</a></h2>
                <Markdown children={item.description} components={{
                  code(props) {
                    const { children, className, node, ...rest } = props
                    const match = /language-(\w+)/.exec(className || '');
                    return match ? (
                      <SyntaxHighlighter language={match[1]} style={dracula} children={String(children).replace(/\n$/, '')} showLineNumbers={true} />
                    ) : <code {...rest} className={className}>{children}</code >;
                  }
                }} />
              </div>
            </div>
          )
          : ''
  },[searchResults])

  return (
    <div className="mt-20 text-center">
      <h2 className="text-primary font-extrabold text-5xl font-jetbrains">SYMENTIC SEARCH THRU WR17UP$</h2>
      <label className="input input-bordered input-primary flex flex-row justify-center items-center gap-2 mx-20 my-5">
        <input type="text" className="grow" placeholder="Search" value={searchValue}
          onChange={(e) => setSearchValue(e.target.value)} />
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" fill="currentColor" className="w-4 h-4 opacity-70"><path fillRule="evenodd" d="M9.965 11.026a5 5 0 1 1 1.06-1.06l2.755 2.754a.75.75 0 1 1-1.06 1.06l-2.755-2.754ZM10.5 7a3.5 3.5 0 1 1-7 0 3.5 3.5 0 0 1 7 0Z" clipRule="evenodd" /></svg>
      </label>
      <div className="mx-40 md:mx-10 lg:mx-[100px] xl:mx-[200px] 2xl:mx-[300px]">
        {rendered_results}
      </div>
    </div >
  );
}

export default App;
