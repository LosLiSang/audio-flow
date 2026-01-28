#!/bin/bash
# Audio Flow - 自动备份脚本

# 配置
BACKUP_DIR="/backup/audio-flow"
DATE=$(date +%Y%m%d_%H%M%S)
PROJECT_NAME="audio-flow"
GIT_DIR=$(git rev-parse --git-dir 2>/dev/null || echo ".git")

# 函数：创建备份
create_backup() {
    local backup_type=$1
    local backup_path="${BACKUP_DIR}/${backup_type}"
    
    echo "📦 创建 ${backup_type} 备份..."
    
    # 创建备份目录
    mkdir -p "${backup_path}"
    
    # 创建 bundle 备份
    git bundle create "${backup_path}/${PROJECT_NAME}_${DATE}.bundle" --all
    
    # 创建克隆备份
    git clone --bare . "${backup_path}/${PROJECT_NAME}_${DATE}.git"
    
    echo "✅ ${backup_type} 备份完成: ${backup_path}"
}

# 主函数
main() {
    echo "=========================================="
    echo "🔄 Audio Flow 自动备份"
    echo "=========================================="
    echo ""
    
    # 检查是否在 Git 仓库中
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        echo "❌ 错误：不是 Git 仓库"
        exit 1
    fi
    
    # 显示当前状态
    echo "📊 当前状态："
    echo "  分支: $(git branch --show-current)"
    echo "  最近提交: $(git log -1 --pretty=format:'%h - %s')"
    echo "  未提交的文件: $(git status --short | wc -l)"
    echo ""
    
    # 创建备份
    create_backup "daily"
    
    echo ""
    echo "📂 备份位置：${BACKUP_DIR}"
    echo "📦 Bundle: ${BACKUP_DIR}/daily/${PROJECT_NAME}_${DATE}.bundle"
    echo "📦 裸仓库: ${BACKUP_DIR}/daily/${PROJECT_NAME}_${DATE}.git"
    echo ""
    echo "✅ 备份完成！"
}

# 运行主函数
main "$@"
