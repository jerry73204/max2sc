#!/usr/bin/env python3
"""
Extract OSC commands from speaker configuration files
"""

import sys
from pathlib import Path
from collections import defaultdict
from typing import Dict, List, Set

def parse_osc_file(file_path: Path) -> Dict[str, List[str]]:
    """Parse OSC commands from a configuration file"""
    osc_commands = defaultdict(list)
    
    with open(file_path, 'r') as f:
        for line in f:
            line = line.strip()
            if line.startswith('/'):
                # Parse OSC command
                parts = line.split()
                if parts:
                    command = parts[0]
                    args = ' '.join(parts[1:])
                    osc_commands[command].append(args)
    
    return osc_commands

def analyze_osc_namespace(directory: Path) -> Dict[str, Set[str]]:
    """Analyze OSC namespace from all configuration files"""
    namespace = defaultdict(set)
    
    # Find all .txt files that might contain OSC commands
    txt_files = list(directory.glob('*.txt'))
    
    for txt_file in txt_files:
        print(f"Analyzing: {txt_file.name}")
        try:
            commands = parse_osc_file(txt_file)
            for cmd in commands:
                # Extract namespace hierarchy
                parts = cmd.split('/')
                for i in range(2, len(parts) + 1):
                    namespace_path = '/'.join(parts[:i])
                    namespace[namespace_path].add(cmd)
        except Exception as e:
            print(f"Error processing {txt_file}: {e}")
    
    return namespace

def generate_osc_report(namespace: Dict[str, Set[str]]) -> str:
    """Generate OSC namespace report"""
    report = []
    report.append("# OSC Namespace Analysis\n")
    
    # Group by top-level namespace
    top_level = defaultdict(set)
    for path in namespace:
        if path.count('/') == 1:  # Top level
            top_level[path] = namespace[path]
    
    for top_path in sorted(top_level.keys()):
        report.append(f"## {top_path}\n")
        
        # Find all sub-paths
        sub_paths = []
        for path in namespace:
            if path.startswith(top_path + '/') and path != top_path:
                sub_paths.append(path)
        
        # Group by category
        categories = defaultdict(list)
        for path in sorted(sub_paths):
            parts = path.split('/')
            if len(parts) >= 3:
                category = parts[2]
                categories[category].append(path)
        
        for category, paths in sorted(categories.items()):
            report.append(f"### {category}")
            for path in sorted(paths)[:10]:  # Limit to 10 examples
                report.append(f"- `{path}`")
            if len(paths) > 10:
                report.append(f"- ... and {len(paths) - 10} more")
            report.append("")
    
    return '\n'.join(report)

def main():
    if len(sys.argv) != 2:
        print("Usage: python extract_osc_commands.py <directory>")
        sys.exit(1)
    
    directory = Path(sys.argv[1])
    if not directory.exists():
        print(f"Directory not found: {directory}")
        sys.exit(1)
    
    print(f"Analyzing OSC commands in: {directory}\n")
    namespace = analyze_osc_namespace(directory)
    
    # Generate report
    report = generate_osc_report(namespace)
    
    # Save results
    output_dir = Path("analysis_output")
    output_dir.mkdir(exist_ok=True)
    
    with open(output_dir / "osc_namespace.md", 'w') as f:
        f.write(report)
    
    print(f"\nAnalysis complete! Results saved to {output_dir}/osc_namespace.md")
    print(report)

if __name__ == "__main__":
    main()