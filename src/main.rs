use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Confirm, FuzzySelect, Input};
use serde::Deserialize;
use std::error::Error;
use std::fs;
use std::process;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 只生成提交信息，不执行 git commit
    #[arg(short, long)]
    dry_run: bool,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Config {
    pub types: Option<Vec<CommitType>>,
    pub scopes: Option<Vec<String>>,
    pub emoji: Option<bool>,
    pub strict_scope: Option<bool>,
    pub max_header_length: Option<usize>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CommitType {
    pub name: String,
    pub description: String,
    pub emoji: Option<String>,
}

impl Config {
    pub fn default_config() -> Self {
        Self {
            types: Some(vec![
                CommitType { name: "feat".to_string(), description: "新功能".to_string(), emoji: Some("✨".to_string()) },
                CommitType { name: "fix".to_string(), description: "Bug修复".to_string(), emoji: Some("🐛".to_string()) },
                CommitType { name: "docs".to_string(), description: "文档修改".to_string(), emoji: Some("📝".to_string()) },
                CommitType { name: "style".to_string(), description: "格式调整".to_string(), emoji: Some("🎨".to_string()) },
                CommitType { name: "refactor".to_string(), description: "代码重构".to_string(), emoji: Some("♻️".to_string()) },
                CommitType { name: "perf".to_string(), description: "性能优化".to_string(), emoji: Some("⚡".to_string()) },
                CommitType { name: "test".to_string(), description: "测试代码".to_string(), emoji: Some("✅".to_string()) },
                CommitType { name: "chore".to_string(), description: "构建工具".to_string(), emoji: Some("🔧".to_string()) },
            ]),
            scopes: Some(vec![
                "core".to_string(),
                "ui".to_string(),
                "api".to_string(),
                "db".to_string(),
                "auth".to_string(),
            ]),
            emoji: Some(false),
            strict_scope: Some(false),
            max_header_length: Some(100),
        }
    }
}

fn load_config() -> Config {
    let files = ["cmg.toml", "cmg.json"];
    for file in files {
        let Ok(content) = fs::read_to_string(file) else { continue };
        let parsed = match file {
            f if f.ends_with(".toml") => toml::from_str(&content).ok(),
            f if f.ends_with(".json") => serde_json::from_str(&content).ok(),
            _ => continue,
        };
        if let Some(cfg) = parsed { return cfg; }
    }
    Config::default_config()
}

fn is_git_repo() -> bool {
    process::Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn run() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let cfg = load_config();
    let theme = ColorfulTheme::default();

    println!("📝 Git 规范提交工具");
    println!("遵循 Conventional Commits 规范\n");

    if !is_git_repo() {
        eprintln!("❌ 当前目录不是 Git 仓库！");
        process::exit(1);
    }

    let default_cfg = Config::default_config();
    let types = cfg.types.or(default_cfg.types).unwrap();
    let scope_list = cfg.scopes.or(default_cfg.scopes).unwrap_or_default();
    let use_emoji = cfg.emoji.unwrap_or(false);
    let strict_scope = cfg.strict_scope.unwrap_or(false);
    let max_len = cfg.max_header_length.or(default_cfg.max_header_length).unwrap();

    // 计算最大名称长度，实现冒号对齐
    let max_name_len = types.iter().map(|t| t.name.len()).max().unwrap_or(0);

    // 提交类型：冒号紧贴 + 对齐
    let type_options: Vec<String> = types
        .iter()
        .map(|t| {
            let name_part = format!("{:<width$}", format!("{}:", t.name), width = max_name_len + 1);
            if use_emoji {
                format!("{} {} {}", t.emoji.as_deref().unwrap_or(""), name_part, t.description)
            } else {
                format!("{} {}", name_part, t.description)
            }
        })
        .collect();

    let type_idx = FuzzySelect::with_theme(&theme)
        .with_prompt("选择提交类型")
        .items(&type_options)
        .default(0)
        .interact()?;
    let selected = &types[type_idx];

    // 作用域：strict_scope = true 时增加【不填（空）】选项，默认选中
    let scope = if !scope_list.is_empty() {
        let mut items = Vec::new();

        if strict_scope {
            items.insert(0, "不填（空）".to_string());
        } else {
            items.push("自定义 scope".to_string());
        }

        items.extend(scope_list.iter().cloned());

        let idx = FuzzySelect::with_theme(&theme)
            .with_prompt("选择作用域")
            .items(&items)
            .default(0)
            .interact()?;

        if strict_scope {
            if idx == 0 {
                String::new()
            } else {
                items[idx].clone()
            }
        } else {
            if idx == 0 {
                let input = Input::<String>::with_theme(&theme)
                    .with_prompt("输入自定义 scope")
                    .allow_empty(true)
                    .interact()?;
                input.trim().to_string()
            } else {
                items[idx].clone()
            }
        }
    } else {
        Input::<String>::with_theme(&theme)
            .with_prompt("输入 scope")
            .allow_empty(true)
            .interact()?
            .trim().to_string()
    };

    // 简短描述（强制不能为空，直接回车红色提示）
    let subject = loop {
        let s = Input::<String>::with_theme(&theme)
            .with_prompt(format!("输入简短描述（最大 {} 字符）", max_len))
            .interact()?;

        let t = s.trim();
        if t.is_empty() {
            println!("\x1b[31m❌ 错误：简短描述不能为空\x1b[0m");
            continue;
        }
        if t.len() > max_len {
            println!("\x1b[31m❌ 错误：长度不能超过 {} 字符\x1b[0m", max_len);
            continue;
        }
        break t.to_string();
    };

    // 详细描述
    let body = Input::<String>::with_theme(&theme)
        .with_prompt("详细描述（可选，使用 | 即可换行）")
        .allow_empty(true)
        .interact()?
        .replace('|', "\n");

    // ==============================
    // Issue 关闭功能：选择关键词 + 输入编号
    // ==============================
    let issue_keywords = ["Close", "Fix", "Resolve"];
    let default_key = if selected.name == "fix" { 1 } else { 0 };

    let issue_closure = Input::<String>::with_theme(&theme)
        .with_prompt("关闭的 Issues 编号（可选，直接回车跳过）")
        .allow_empty(true)
        .interact()?
        .trim().to_string();

    let issue_part = if !issue_closure.is_empty() {
        let key_idx = FuzzySelect::with_theme(&theme)
            .with_prompt("选择关闭方式")
            .items(&issue_keywords)
            .default(default_key)
            .interact()?;

        let keyword = issue_keywords[key_idx];
        format!("\n\n{} #{}", keyword, issue_closure)
    } else {
        String::new()
    };

    // 破坏性变更
    let breaking = Confirm::with_theme(&theme)
        .with_prompt("是否包含破坏性变更？")
        .default(false)
        .interact()?;

    // 生成最终提交信息
    let mut msg = String::new();

    if use_emoji {
        if let Some(emoji) = &selected.emoji {
            msg.push_str(emoji);
            msg.push(' ');
        }
    }

    msg.push_str(&selected.name);

    if !scope.is_empty() {
        msg.push_str(&format!("({})", scope));
    }

    if breaking {
        msg.push('!');
    }

    msg.push_str(&format!(": {}", subject));

    if !body.is_empty() {
        msg.push_str(&format!("\n\n{}", body));
    }

    msg.push_str(&issue_part);

    if breaking {
        msg.push_str("\n\nBREAKING CHANGE: 破坏性变更");
    }

    // 预览
    println!("\n📄 提交信息预览：");
    println!("{}\n", msg);

    if args.dry_run {
        println!("✅ dry-run 模式，未提交");
        return Ok(());
    }

    // 确认提交
    let confirm = Confirm::with_theme(&theme)
        .with_prompt("确定提交？")
        .default(true)
        .interact()?;

    if !confirm {
        println!("🚫 已取消提交");
        return Ok(());
    }

    // 执行提交
    let status = process::Command::new("git")
        .args(["commit", "-m", &msg])
        .status()?;

    if status.success() {
        println!("🎉 提交成功！");
    } else {
        eprintln!("❌ 提交失败，请先 git add 暂存文件");
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("❌ 错误：{}", e);
        process::exit(1);
    }
}
