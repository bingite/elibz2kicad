import React, { useState, useEffect } from 'react';
import ReactMarkdown, { defaultUrlTransform } from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { Card, Typography, Divider } from 'antd';

// 定义 ReactMarkdown 组件的属性类型
interface ReactMarkdownProps {
  node?: any;
  inline?: boolean;
  className?: string;
  children?: React.ReactNode;
}

/**
 * @brief Markdown查看器组件
 * @details 用于显示Markdown文件内容，支持图片显示
 */
const MarkdownViewer: React.FC = () => {
  const [markdownContent, setMarkdownContent] = useState('');

  /**
   * @brief 组件挂载时加载示例Markdown内容
   */
  useEffect(() => {
    // 从public目录加载示例markdown文件
    fetch('/example/elibz2kicad使用教程.md')
      .then(response => response.text())
      .then(text => setMarkdownContent(text))
      .catch(error => {
        console.error('加载Markdown文件失败:', error);
        // 如果加载失败，使用默认内容
        const defaultContent = `
# Markdown查看器

无法加载示例文件，请确保文件存在。

## 使用说明

将Markdown文件放置在public目录下，即可在应用中查看。
        `;
        setMarkdownContent(defaultContent);
      });
  }, []);

  // 自定义URL转换函数，用于处理相对路径图片
  const customUrlTransform = (url: string): string => {
    // 如果是相对路径，则添加/example前缀
    if (url.startsWith('./') || !url.includes('://')) {
      return `/example/${url}`;
    }
    // 否则使用默认的URL转换函数
    return defaultUrlTransform(url);
  };

  return (
    <div style={{ padding: '20px' }}>
      <Card>
        <Typography.Title level={2}>Markdown查看器</Typography.Title>
        <Divider />
        <div style={{ textAlign: 'left' }}>
            <ReactMarkdown 
              urlTransform={customUrlTransform}
              remarkPlugins={[remarkGfm]}
              components={{
                img: ({ node, ...props }) => (
                  <img 
                    {...props} 
                    style={{ maxWidth: '100%', height: 'auto' }} 
                  />
                ),
                code({node, inline, className, children, ...props}: ReactMarkdownProps & { inline?: boolean }) {
                  const match = /language-(\w+)/.exec(className || '')
                  return !inline && match ? (
                    <code className={className} style={{ backgroundColor: '#f5f5f5', padding: '2px 4px', borderRadius: '4px', overflowX: 'auto', display: 'block' }} {...props}>
                      {children}
                    </code>
                  ) : (
                    <code className={className} {...props}>
                      {children}
                    </code>
                  )
                }
              }}
            >
              {markdownContent}
            </ReactMarkdown>
          </div>
      </Card>
    </div>
  );
};

export default MarkdownViewer;