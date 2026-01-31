# CLAUDE.md - Shuati AI 项目记忆文档

> 本文档用于保持开发语境连续性，记录技术决策、核心API结构及已知问题。

## 项目概述

**Shuati AI** - 基于 Tauri 2.0 的现代跨平台 AI 刷题应用，支持复杂 LaTeX 公式与多媒体资源处理。

- **技术栈**: Tauri 2.0 + Rust + React + TypeScript + SQLite
- **前端渲染**: KaTeX (轻量级 LaTeX 渲染)
- **UI框架**: Shadcn UI + Tailwind CSS
- **AI集成**: OpenAI Structured Outputs (计划中)

## 核心架构决策

### 1. 数据库设计

采用 SQLite 作为主数据库，启用 WAL 模式提升并发性能。

```sql
-- 核心表结构
questions: 题目存储（支持多种题型）
question_attempts: 答题记录与评分
mistake_collections: 错题集关联表
study_progress: 学习进度追踪
knowledge_tags: 知识点标签系统
```

**已配置插件**:
- `tauri-plugin-sql` - SQLite 数据库访问
- `tauri-plugin-fs` - 文件系统操作
- `tauri-plugin-dialog` - 原生对话框

### 2. 路径处理策略

跨平台路径兼容性处理：

```rust
// macOS: $APP_DATA/shuati/
// Windows: AppData/Roaming/com.nowaywastaken.tauri-app/
```

**待实现**: asset 协议安全加载本地图片资源。

### 3. AI 生成流程

题目生成采用 "先解析、后刷题" 原则：

1. **Generator Agent**: 基于 Markdown 文档生成结构化题目
2. **Verifier Agent**: 验证 LaTeX 公式闭合性、答案逻辑自洽
3. **Formatter Agent**: 标准化输出格式

**当前状态**: Mock 实现（模拟 1.5s 延迟）
**下一步**: 接入 OpenAI Batch API 实现经济高效的批量处理

## 已知问题与限制

### 前端渲染

- KaTeX 不支持所有 LaTeX 特性，复杂公式可能渲染失败
- 已采用 `React.memo` 优化长文本滚动性能

### 文件处理

- 图片资源尚未实现 asset 协议转换
- 大型图片应考虑 Base64 缓存于 SQLite BLOB 字段

### 数据库

- WAL 模式需要在 Rust 后端显式启用
- 高频写入操作需优化批量插入性能

## API 结构

### 核心类型定义

```typescript
// src/lib/db.ts
interface Question {
  id: number;
  question_type: 'multiple_choice' | 'fill_in_the_blank' | 'essay';
  stem: string;              // Markdown 格式题干
  options: string;           // JSON 序列化选项数组
  reference_answer: string;  // LaTeX 格式参考答案
  detailed_analysis: string; // JSON 序列化解析步骤
  media_refs: string;        // JSON 序列化媒体路径
  knowledge_tags: string;    // JSON 序列化知识点标签
  difficulty: number;        // 1-5 难度等级
  created_at: string;
}

interface QuestionAttempt {
  id: number;
  question_id: number;
  user_answer: string;
  is_correct: boolean;
  confidence_score: number;  // AI 评分置信度
  time_spent_seconds: number;
  created_at: string;
}
```

### Rust 命令

```rust
// src-tauri/src/lib.rs 计划添加:
- batch_import_questions() -> 批量导入题目
- save_attempt() -> 保存答题记录
- get_mistakes_by_tag() -> 按标签查询错题
- analyze_study_patterns() -> 学习模式分析（DuckDB）
```

## 下一步开发重点

1. **高优先级**: 配置 WAL 模式 + 完整数据模型
2. **高优先级**: 答题评分系统实现
3. **中优先级**: OpenAI API 集成（需要 API Key 配置界面）
4. **中优先级**: 练习模式 UI（答题卡、计时器）
5. **低优先级**: DuckDB 分析引擎（当题库规模 > 1000 题时启用）

## 开发环境配置

```bash
# 启动开发服务器
npm run tauri dev

# 构建生产版本
npm run tauri build
```

**注意事项**:
- macOS 需要 Xcode 命令行工具
- Windows 需要 WebView2 运行时
- Rust 版本需 >= 1.70

## 最近更新

- 2025-01-31: 初始化项目架构，完成基础 UI 和 Mock AI 生成
- 2026-01-31: 
  - 创建 CLAUDE.md 持续记忆文档
  - 配置 SQLite WAL 模式（Rust 后端）
  - 实现完整数据模型：questions, question_attempts, mistake_collections
  - 集成 OpenAI Structured Outputs 支持（含多代理流水线框架）
  - 添加答题和评分系统（练习模式）
  - 实现错题集自动记录功能
  - 更新前端界面：练习模式、统计面板、题目卡片增强
  - 添加 OpenAI API Key 配置支持

## 当前架构状态

### 后端 (Rust)
- ✅ SQLite WAL 模式启用
- ✅ 完整表结构：questions, question_attempts, mistake_collections
- ✅ Rust 命令：init_database, batch_import_questions, get_all_questions, save_attempt, get_mistakes_by_tag

### 前端 (React + TypeScript)
- ✅ 练习模式界面（PracticeMode）
- ✅ 答题评分系统（选择题、填空题、简答题）
- ✅ 题目卡片增强（难度、标签、解析显示）
- ✅ Dashboard 统计面板
- ✅ Settings 页面（API Key 配置 UI）

### AI 集成
- ✅ Structured Outputs JSON Schema
- ✅ 多代理流水线框架（Generator → Verifier → Formatter）
- ✅ Batch API 支持框架（待完善文件上传）
- ✅ 真正的 API 调用（需要配置 API Key）

## 下一步

1. 添加图片资源处理（asset 协议）
2. DuckDB 分析引擎（当题库 > 1000 题时）
3. 间隔重复算法（Spaced Repetition）
4. 练习模式增强（答题卡、计时器）
