<string ,="true" string="true">;
  };
  
  /**
   * 插件配置
   */
  plugins?: string[];
  
  /**
   * 其他配置选项
   */
  [key: string]: any;
}

/**
 * 定义 HXO 配置
 * @param config 配置选项
 * @returns 配置对象
 */
export function defineConfig(config: NargoConfig = {}): NargoConfig {
  return config;
}

/**
 * HXO 核心工具
 */
export const nargo = {
  /**
   * 版本号
   */
  version: '0.1.0',
  
  /**
   * 定义配置
   */
  defineConfig,
  
  /**
   * 检查配置有效性
   * @param config 配置对象
   * @returns 是否有效
   */
  validateConfig: (config: NargoConfig): boolean => {
    return typeof config === 'object' && config !== null;
  },
  
  /**
   * SSR 相关功能
   */
  ssr: {
    /**
     * 渲染组件到 HTML 字符串
     * @param renderFn 渲染函数
     * @param context 组件上下文
     * @returns 生成的 HTML 字符串
     */
    renderToString: (renderFn: (ctx: any) => string, context: any = {}): string => {
      return renderFn(context);
    },
    
    /**
     * 为 hydration 准备状态
     * @param state 应用状态
     * @returns 序列化的状态字符串
     */
    serializeState: (state: any): string => {
      return JSON.stringify(state);
    },
    
    /**
     * 从 HTML 中提取状态
     * @param html HTML 字符串
     * @returns 提取的状态对象
     */
    extractState: (html: string): any => {
      const match = html.match(/<script type="nargo\/state">(.*?)<\/script>/s);
      if (match) {
        try {
          return JSON.parse(match[1]);
        } catch (e) {
          console.error('Failed to parse state:', e);
          return {};
        }
      }
      return {};
    }
  }
};

export default nargo;
</string>
