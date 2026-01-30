import React, { useEffect, useRef } from 'react'
import ReactMarkdown from 'react-markdown'
import remarkMath from 'remark-math'
import rehypeKatex from 'rehype-katex'
import 'katex/dist/katex.min.css'

interface MarkdownRendererProps {
  content: string
  className?: string
}

export function MarkdownRenderer({ content, className = '' }: MarkdownRendererProps) {
  const containerRef = useRef<HTMLDivElement>(null)

  // 重新渲染 KaTeX
  useEffect(() => {
    if (containerRef.current) {
      const katexElements = containerRef.current.querySelectorAll('.katex')
      katexElements.forEach((el) => {
        // KaTeX 自动渲染，无需手动处理
      })
    }
  }, [content])

  return (
    <div ref={containerRef} className={className}>
      <ReactMarkdown
        remarkPlugins={[remarkMath]}
        rehypePlugins={[rehypeKatex]}
        components={{
          // 自定义组件映射
          code({ node, inline, className, children, ...props }) {
            return inline ? (
              <code className="bg-muted px-1.5 py-0.5 rounded text-sm" {...props}>
                {children}
              </code>
            ) : (
              <pre className="bg-muted p-4 rounded-lg overflow-x-auto">
                <code {...props}>{children}</code>
              </pre>
            )
          },
          math({ node, inline, children, ...props }) {
            // LaTeX 公式处理
            return (
              <span className={inline ? 'inline-math' : 'block-math'}>
                {children}
              </span>
            )
          },
        }}
      >
        {content}
      </ReactMarkdown>
    </div>
  )
}