import { useState, useEffect } from 'react'
import './App.css'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core';
import { Button, Space, Typography, Divider, Layout, Modal } from 'antd';
import { FolderOpenOutlined, FileOutlined, PlayCircleOutlined } from '@ant-design/icons';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import Sidebar from './components/Sidebar';
import MarkdownViewer from './components/MarkdownViewer';


const { Header, Content, Footer, Sider } = Layout;

function App() {
  const [elibzFiles, setElibzFiles] = useState<string[]>([])
  const [outputDir, setOutputDir] = useState('')
  //const [kicadSymFile, setKicadSymFile] = useState('')
  const [collapsed, setCollapsed] = useState(false);
  const [isConverted, setIsConverted] = useState(false);
  const [conversionResult, setConversionResult] = useState('');
  const [isModalVisible, setIsModalVisible] = useState(false);

  const handleConvert = async () => {
    setIsConverted(false);
    setConversionResult('');

    try {
      // 为每个文件分别调用后端API
      const results = [];
      for (const file of elibzFiles) {
        const result = await invoke('process_elib_file', { elibzFile: file, outputDir: outputDir, kicadSymFile: "" });
        results.push(`${file}: ${result}`);
      }
      
      const finalResult = results.join('\n\n');
      setConversionResult(finalResult);
      setIsConverted(true);
      setIsModalVisible(true);
    } catch (error) {
      Modal.info({
        title: '转换结果',
        content: `转换失败: ${error}`,
      });
    }
  }

  useEffect(() => {
    if (isConverted) {
      const timer = setTimeout(() => {
        setIsConverted(false);
      }, 3000);
      return () => clearTimeout(timer);
    }
  }, [isConverted]);

  /**
   * @brief 选择.elibz文件
   * @details 打开文件选择对话框，选择.elibz文件并获取其绝对路径
   */
  const pickElibzFiles = async () => {
    const selected = await open({
      multiple: true,
      filters: [{
        name: 'Elibz Files',
        extensions: ['elibz']
      }]
    });
    if (selected) {
      // 确保获取的是绝对路径
      setElibzFiles(selected as string[]);
    }
  };

  /**
   * @brief 选择输出目录
   * @details 打开目录选择对话框，选择输出目录并获取其绝对路径
   */
  const pickOutputDir = async () => {
    const selected = await open({
      directory: true,
      multiple: false
    });
    if (selected) {
      // 确保获取的是绝对路径
      setOutputDir(selected as string);
    }
  };

  /**
   * @brief 选择.kicad_sym文件
   * @details 打开文件选择对话框，选择.kicad_sym文件并获取其绝对路径
   *
   * NOTE: 此功能暂时隐藏
   */
  // const pickKicadSymFile = async () => {
  //   const selected = await open({
  //     multiple: false,
  //     filters: [{
  //       name: 'KiCad Symbol Files',
  //       extensions: ['kicad_sym']
  //     }]
  //   });
  //   if (selected) {
  //     // 确保获取的是绝对路径
  //     setKicadSymFile(selected as string);
  //   }
  // };

  return (
    <Router>
      <Layout style={{ minHeight: '100vh' }}>
        <Sider collapsible collapsed={collapsed} onCollapse={setCollapsed}>
          <Sidebar />
        </Sider>
        <Layout>
          <Header style={{ padding: 0, background: '#fff' }} />
          <Content style={{ margin: '24px 16px 0' }}>
            <Routes>
              <Route path="/" element={
                <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100%' }}>
                  <Space direction="vertical" style={{ width: '100%', maxWidth: '600px' }} size="large">
                    <div>
                      <Typography.Text strong>选择.elibz文件:</Typography.Text>
                      <div style={{ display: 'flex', alignItems: 'center', marginTop: '8px' }}>
                        <Button icon={<FileOutlined />} onClick={pickElibzFiles}>
                          选择文件
                        </Button>
                        <div style={{ marginLeft: '12px' }}>
                          {elibzFiles.map((file, index) => (
                            <Typography.Text key={index} ellipsis={{ tooltip: file }} style={{ display: 'block' }}>
                              {file}
                            </Typography.Text>
                          ))}
                        </div>
                      </div>
                    </div>
                    <Divider />
                    <div>
                      <Typography.Text strong>选择输出封装到目录:</Typography.Text>
                      <div style={{ display: 'flex', alignItems: 'center', marginTop: '8px' }}>
                        <Button icon={<FolderOpenOutlined />} onClick={pickOutputDir}>
                          选择目录
                        </Button>
                        <Typography.Text style={{ marginLeft: '12px' }} ellipsis={{ tooltip: outputDir }}>
                          {outputDir}
                        </Typography.Text>
                      </div>
                    </div>
                    <Divider />
                    {/* <div>
                      <Typography.Text strong>选择输出符号到.kicad_sym文件:</Typography.Text>
                      <div style={{ display: 'flex', alignItems: 'center', marginTop: '8px' }}>
                        <Button icon={<FileOutlined />} onClick={pickKicadSymFile}>
                          选择文件
                        </Button>
                        <Typography.Text style={{ marginLeft: '12px' }} ellipsis={{ tooltip: kicadSymFile }}>
                          {kicadSymFile}
                        </Typography.Text>
                      </div>
                    </div> */}
                    <Divider />
                    <div style={{ textAlign: 'center' }}>
                
                        <Button type="primary" icon={<PlayCircleOutlined />} onClick={handleConvert} size="large">
                          进行转换
                        </Button>
                      
                    </div>
                    <Modal
                      title="转换结果"
                      open={isModalVisible}
                      onOk={() => setIsModalVisible(false)}
                      onCancel={() => setIsModalVisible(false)}
                      width={800}
                      okText="确定"
                      cancelText="取消"
                    >
                      {conversionResult && (
                        <div style={{ marginTop: '20px', padding: '10px', backgroundColor: '#f6ffed', border: '1px solid #b7eb8f', borderRadius: '4px' }}>
                          <Typography.Paragraph style={{ whiteSpace: 'pre-wrap' }}>{conversionResult}</Typography.Paragraph>
                        </div>
                      )}
                    </Modal>
                    <Footer style={{ textAlign: 'center' }}>
                      <Typography.Paragraph style={{ textAlign: 'center', marginTop: '20px' }}>
                        请选择相应的文件和目录，然后点击"进行转换"按钮
                      </Typography.Paragraph>
                    </Footer>
                  </Space>
                </div>
              } />
              <Route path="/markdown" element={<MarkdownViewer />} />
            </Routes>
          </Content>

        </Layout>
      </Layout>
    </Router>
  )
}

export default App
