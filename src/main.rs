use clap::{Args, Parser, Subcommand};
use std::io::{self, Write};
use std::path::PathBuf;
use anyhow::{Context, Result};

use reminex::db::Database;
use reminex::indexer::{scan_idxs_with_metadata, scan_idxs};
use reminex::searcher::{search_from_input, build_tree, print_tree, SearchConfig};

fn main() {
    if let Err(e) = run() {
        eprintln!("错误: {:#}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let app = App::parse();
    
    match app.commands {
        Commands::Index(args) | Commands::I(args) => {
            handle_index_command(args)?;
        },
        Commands::Search(args) | Commands::S(args) => {
            handle_search_command(args)?;
        }
    }
    
    Ok(())
}

fn handle_index_command(args: IndexArgs) -> Result<()> {
    // 确定根目录路径
    let root_path = args.path.unwrap_or_else(|| PathBuf::from("./"));
    
    if !root_path.exists() {
        anyhow::bail!("路径不存在: {}", root_path.display());
    }
    
    // 确定数据库路径
    let db_path = args.db.unwrap_or_else(|| {
        root_path.join(".reminex.db")
    });
    
    println!("📁 索引目录: {}", root_path.display());
    println!("💾 数据库文件: {}", db_path.display());
    
    // 初始化或打开数据库
    let db = if db_path.exists() && !args.full {
        println!("📂 使用现有数据库");
        Database::new(&db_path)
    } else {
        if args.full {
            println!("🔄 执行全量重建");
            // 删除旧数据库
            if db_path.exists() {
                std::fs::remove_file(&db_path)
                    .context("无法删除旧数据库")?;
            }
        } else {
            println!("🆕 创建新数据库");
        }
        Database::init(&db_path)?
    };
    
    // 执行扫描
    let batch_size = args.batch_size.unwrap_or(5000);
    
    println!("🚀 开始扫描...");
    println!("   批量大小: {}", batch_size);
    
    let duration = if args.no_metadata {
        println!("   模式: 快速扫描（无元数据）");
        scan_idxs(&root_path, &db, batch_size)?
    } else {
        println!("   模式: 完整扫描（含元数据）");
        scan_idxs_with_metadata(&root_path, &db, batch_size)?
    };
    
    // 统计信息
    let count = db.batch_operation(|conn| {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM files",
            [],
            |row| row.get(0)
        )?;
        Ok(count)
    })?;
    
    println!("\n✅ 索引完成！");
    println!("   耗时: {:.2}s", duration.as_secs_f64());
    println!("   文件数: {}", count);
    println!("   速度: {:.0} 文件/秒", count as f64 / duration.as_secs_f64());
    
    Ok(())
}

fn handle_search_command(args: SearchArgs) -> Result<()> {
    // 确定数据库路径
    let db_path = args.db.clone().unwrap_or_else(|| PathBuf::from("./.reminex.db"));
    
    if !db_path.exists() {
        anyhow::bail!("数据库文件不存在: {}\n请先运行索引命令创建数据库", db_path.display());
    }
    
    let db = Database::new(&db_path);
    
    // 配置搜索参数
    let config = SearchConfig {
        max_results: args.limit.unwrap_or(2000),
        search_in_path: !args.name_only,
        case_sensitive: args.case_sensitive,
    };
    
    // 如果提供了关键词，直接搜索
    if let Some(ref keywords) = args.keywords {
        perform_search(&db, keywords, &config, &args)?;
        return Ok(());
    }
    
    // 交互模式
    println!("🔍 reminex 搜索模式");
    println!("   数据库: {}", db_path.display());
    println!("   输入关键词搜索，多个关键词用 ; 或空格分隔");
    println!("   输入 :q 退出\n");
    
    loop {
        print!("搜索> ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if input.is_empty() {
            continue;
        }
        
        if input == ":q" || input == "exit" || input == "quit" {
            println!("再见！");
            break;
        }
        
        perform_search(&db, input, &config, &args)?;
    }
    
    Ok(())
}

fn perform_search(db: &Database, input: &str, config: &SearchConfig, args: &SearchArgs) -> Result<()> {
    let results = search_from_input(db, input, config)?;
    
    if results.is_empty() {
        println!("\n❌ 未找到任何结果\n");
        return Ok(());
    }
    
    for (keyword, items) in results {
        if items.is_empty() {
            println!("\n「{}」未找到任何结果", keyword);
            continue;
        }
        
        println!("\n「{}」找到 {} 项结果：", keyword, items.len());
        
        if args.tree {
            // 树形显示
            let root_name = args.root_name.as_deref().unwrap_or("搜索结果");
            
            let tree = build_tree(&items, root_name);
            println!();
            print_tree(&tree);
        } else {
            // 列表显示
            println!();
            for item in &items {
                println!("  {}", item.path);
            }
        }
        println!();
    }
    
    Ok(())
}

#[derive(Parser)]
#[command(name = "reminex")]
#[command(about = "快速文件索引和搜索工具", long_about = None)]
#[command(version)]
struct App {
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "索引文件 (index)")]
    Index(IndexArgs),
    
    #[command(about = "索引文件 (index 简写)")]
    I(IndexArgs),
    
    #[command(about = "搜索文件 (search)")]
    Search(SearchArgs),
    
    #[command(about = "搜索文件 (search 简写)")]
    S(SearchArgs),
}

#[derive(Args, Clone)]
struct IndexArgs {
    #[arg(short, long, help = "要索引的目录路径")]
    path: Option<PathBuf>,
    
    #[arg(short, long, help = "数据库文件路径")]
    db: Option<PathBuf>,
    
    #[arg(short, long, help = "全量重建索引（删除旧数据）")]
    full: bool,
    
    #[arg(short = 'n', long, help = "快速模式（不扫描文件元数据）")]
    no_metadata: bool,
    
    #[arg(short, long, help = "批量写入大小")]
    batch_size: Option<usize>,
}

#[derive(Args, Clone)]
struct SearchArgs {
    #[arg(help = "搜索关键词（可选，不提供则进入交互模式）")]
    keywords: Option<String>,
    
    #[arg(short, long, help = "数据库文件路径")]
    db: Option<PathBuf>,
    
    #[arg(short, long, help = "结果数量限制", default_value = "2000")]
    limit: Option<usize>,
    
    #[arg(short = 't', long, help = "树形显示结果")]
    tree: bool,
    
    #[arg(short = 'N', long, help = "仅搜索文件名（不搜索路径）")]
    name_only: bool,
    
    #[arg(short = 'c', long, help = "区分大小写")]
    case_sensitive: bool,
    
    #[arg(long, help = "树形显示的根目录名称", default_value = "搜索结果")]
    root_name: Option<String>,
}