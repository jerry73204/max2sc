#!/usr/bin/env python3
"""
Analyze Max patches to extract object usage and mappings
"""

import json
import os
import sys
from pathlib import Path
from collections import defaultdict, Counter
from typing import Dict, List, Set, Any

def extract_objects_from_patch(patch_path: Path) -> Dict[str, List[Dict[str, Any]]]:
    """Extract all objects from a Max patch file"""
    objects = defaultdict(list)
    
    try:
        with open(patch_path, 'r') as f:
            patch_data = json.load(f)
    except Exception as e:
        print(f"Error reading {patch_path}: {e}")
        return objects
    
    def traverse_patcher(patcher_data, parent_path=""):
        """Recursively traverse patcher structure"""
        if not isinstance(patcher_data, dict):
            return
            
        # Get boxes (objects) in this patcher
        boxes = patcher_data.get('boxes', [])
        for box in boxes:
            if not isinstance(box, dict) or 'box' not in box:
                continue
                
            box_data = box['box']
            maxclass = box_data.get('maxclass', '')
            
            # Extract object info
            obj_info = {
                'class': maxclass,
                'text': box_data.get('text', ''),
                'id': box_data.get('id', ''),
                'numinlets': box_data.get('numinlets', 0),
                'numoutlets': box_data.get('numoutlets', 0),
                'patching_rect': box_data.get('patching_rect', []),
                'patch_file': str(patch_path),
                'parent_path': parent_path
            }
            
            # Special handling for newobj
            if maxclass == 'newobj' and 'text' in box_data:
                text = box_data['text']
                # Extract the actual object name from text
                parts = text.split()
                if parts:
                    actual_obj = parts[0]
                    obj_info['actual_class'] = actual_obj
                    obj_info['args'] = ' '.join(parts[1:])
                    objects[actual_obj].append(obj_info)
            else:
                objects[maxclass].append(obj_info)
            
            # Check for subpatchers
            if 'patcher' in box_data:
                subpatcher_name = box_data.get('text', f'subpatcher_{box_data.get("id", "unknown")}')
                traverse_patcher(box_data['patcher'], f"{parent_path}/{subpatcher_name}")
    
    # Start traversal from root patcher
    if 'patcher' in patch_data:
        traverse_patcher(patch_data['patcher'])
    
    return objects

def analyze_directory(directory: Path) -> Dict[str, Any]:
    """Analyze all Max patches in a directory"""
    all_objects = defaultdict(list)
    patch_count = 0
    
    # Find all .maxpat files
    for patch_file in directory.rglob('*.maxpat'):
        patch_count += 1
        print(f"Analyzing: {patch_file}")
        
        objects = extract_objects_from_patch(patch_file)
        for obj_class, instances in objects.items():
            all_objects[obj_class].extend(instances)
    
    # Create summary statistics
    summary = {
        'total_patches': patch_count,
        'unique_objects': len(all_objects),
        'object_counts': {k: len(v) for k, v in all_objects.items()},
        'spatial_objects': {},
        'multichannel_objects': {},
        'spat5_objects': {},
        'routing_objects': {},
        'audio_io_objects': {}
    }
    
    # Categorize objects
    for obj_class, instances in all_objects.items():
        if obj_class.startswith('mc.'):
            summary['multichannel_objects'][obj_class] = len(instances)
        elif obj_class.startswith('spat5.'):
            summary['spat5_objects'][obj_class] = len(instances)
        elif obj_class in ['dac~', 'adc~', 'ezadc~', 'ezdac~', 'sfplay~', 'sfrecord~']:
            summary['audio_io_objects'][obj_class] = len(instances)
        elif obj_class in ['pan~', 'pan2~', 'pan4~', 'pan8~', 'vbap', 'hoa.']:
            summary['spatial_objects'][obj_class] = len(instances)
        elif obj_class in ['matrix~', 'gate~', 'selector~', 'route', 'router']:
            summary['routing_objects'][obj_class] = len(instances)
    
    return summary, all_objects

def generate_mapping_report(summary: Dict[str, Any], all_objects: Dict[str, List]) -> str:
    """Generate a detailed mapping report"""
    report = []
    report.append("# Max MSP to SuperCollider Object Mapping Analysis\n")
    report.append(f"## Summary\n")
    report.append(f"- Total patches analyzed: {summary['total_patches']}")
    report.append(f"- Unique object types: {summary['unique_objects']}\n")
    
    # Top objects by usage
    report.append("## Most Used Objects\n")
    sorted_objects = sorted(summary['object_counts'].items(), key=lambda x: x[1], reverse=True)
    for i, (obj, count) in enumerate(sorted_objects[:20]):
        report.append(f"{i+1}. `{obj}`: {count} instances")
    
    # Spatial audio objects
    report.append("\n## Spatial Audio Objects\n")
    if summary['spat5_objects']:
        report.append("### SPAT5 Objects")
        for obj, count in sorted(summary['spat5_objects'].items()):
            report.append(f"- `{obj}`: {count} instances")
            # Add example usage
            if obj in all_objects and all_objects[obj]:
                example = all_objects[obj][0]
                if example.get('args'):
                    report.append(f"  - Example: `{obj} {example['args']}`")
    
    if summary['spatial_objects']:
        report.append("\n### Native Spatial Objects")
        for obj, count in sorted(summary['spatial_objects'].items()):
            report.append(f"- `{obj}`: {count} instances")
    
    # Multichannel objects
    report.append("\n## Multichannel Objects\n")
    for obj, count in sorted(summary['multichannel_objects'].items()):
        report.append(f"- `{obj}`: {count} instances")
    
    # Audio I/O
    report.append("\n## Audio I/O Objects\n")
    for obj, count in sorted(summary['audio_io_objects'].items()):
        report.append(f"- `{obj}`: {count} instances")
    
    # Routing objects
    report.append("\n## Routing Objects\n")
    for obj, count in sorted(summary['routing_objects'].items()):
        report.append(f"- `{obj}`: {count} instances")
    
    return '\n'.join(report)

def main():
    if len(sys.argv) != 2:
        print("Usage: python analyze_max_patches.py <directory>")
        sys.exit(1)
    
    directory = Path(sys.argv[1])
    if not directory.exists():
        print(f"Directory not found: {directory}")
        sys.exit(1)
    
    print(f"Analyzing Max patches in: {directory}\n")
    summary, all_objects = analyze_directory(directory)
    
    # Generate report
    report = generate_mapping_report(summary, all_objects)
    
    # Save detailed results
    output_dir = Path("analysis_output")
    output_dir.mkdir(exist_ok=True)
    
    # Save summary
    with open(output_dir / "summary.json", 'w') as f:
        json.dump(summary, f, indent=2)
    
    # Save detailed object list
    detailed_objects = {
        obj_class: [
            {k: v for k, v in inst.items() if k != 'patch_file'}
            for inst in instances[:5]  # Limit to 5 examples per type
        ]
        for obj_class, instances in all_objects.items()
    }
    
    with open(output_dir / "detailed_objects.json", 'w') as f:
        json.dump(detailed_objects, f, indent=2)
    
    # Save report
    with open(output_dir / "mapping_analysis.md", 'w') as f:
        f.write(report)
    
    print(f"\nAnalysis complete! Results saved to {output_dir}/")
    print(report)

if __name__ == "__main__":
    main()