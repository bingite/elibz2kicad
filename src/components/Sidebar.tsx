import React from 'react';
import { Menu } from 'antd';
import { HomeOutlined, FileMarkdownOutlined } from '@ant-design/icons';
import { useNavigate, useLocation } from 'react-router-dom';
import svg from '/elibz2kicad.svg';

/**
 * @brief 侧边栏导航组件
 * @details 提供应用内的页面导航功能
 */
const Sidebar: React.FC = () => {
  const navigate = useNavigate();
  const location = useLocation();

  /**
   * @brief 处理菜单点击事件
   * @param path 导航路径
   */
  const handleMenuClick = (path: string) => {
    navigate(path);
  };

  const items = [
    {
      label: (
        <div style={{ display: 'flex', alignItems: 'center', padding: '16px 0' }}>
          <img src={svg} alt="elibz2kicad" style={{ width: '32px', height: '32px', marginRight: '12px' }} />
          <span style={{ fontWeight: 'bold', color: '#000000' }}>elibz2kicad工具</span>
        </div>
      ),
      key: 'header',
      disabled: true,
      style: { height: 'auto', padding: 0 }
    },
    {
      label: '主界面',
      key: '/',
      icon: <HomeOutlined />,
    },
    {
      label: '使用教程',
      key: '/markdown',
      icon: <FileMarkdownOutlined />,
    }
  ];

  return (
    <Menu
      mode="inline"
      selectedKeys={[location.pathname]}
      items={items}
      onClick={({ key }) => handleMenuClick(key)}
      style={{ height: '100%', borderRight: 0 }}
    />
  );
};

export default Sidebar;