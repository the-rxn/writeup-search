import { ExtraProps } from 'react-markdown';
import SyntaxHighlighter from 'react-syntax-highlighter';
import { docco } from 'react-syntax-highlighter/dist/esm/styles/hljs';
const CodeComponent = (props: React.ClassAttributes<HTMLElement> & React.HTMLAttributes<HTMLElement> & ExtraProps) => {
    const { children, className, node, ...rest } = props
    const codeString = '(num) => num + 1';
    const match = /language-(\w+)/.exec(className || '');
    return match ? (
        <SyntaxHighlighter language={match[1]} style={docco}>
            {codeString}
        </SyntaxHighlighter>
    ) : <code {...rest} className={className}>{children}</code >;
};

export default CodeComponent;

//   <Markdown components={{
//     code(props) {
//       const { children, className, node, ...rest } = props
//       const match = /language-(\w+)/.exec(className || '')
//       return match ? (
//         <SyntaxHighlighter
//           {...rest}
//           PreTag="div"
//           children={String(children).replace(/\n$/, '')}
//           language={match[1]}
//         // style={dark}
//         />
//       ) : (
//         <code {...rest} className={className}>
//           {children}
//         </code>
//       )
//     }
//   }}>{item.description}</Markdown>