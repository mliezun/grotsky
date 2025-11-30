import os
import sys

def parse_lcov(lcov_path):
    coverage = {}
    current_file = None
    
    if not os.path.exists(lcov_path):
        print(f"Error: {lcov_path} not found")
        return

    with open(lcov_path, 'r') as f:
        for line in f:
            line = line.strip()
            if line.startswith('SF:'):
                current_file = line[3:]
                # Only care about src/ files
                if 'src/' not in current_file or 'test/' in current_file or 'target/' in current_file or '.cargo/' in current_file or '.rustup/' in current_file:
                    current_file = None
                    continue
                if current_file not in coverage:
                    coverage[current_file] = {'lines': {}, 'total': 0, 'hit': 0}
            
            if current_file and line.startswith('DA:'):
                parts = line[3:].split(',')
                line_num = int(parts[0])
                count = int(parts[1])
                coverage[current_file]['lines'][line_num] = count
                coverage[current_file]['total'] += 1
                if count > 0:
                    coverage[current_file]['hit'] += 1

    return coverage

def print_gaps(coverage):
    print("Coverage Gaps Analysis:")
    print("=======================")
    
    sorted_files = sorted(coverage.keys(), key=lambda k: coverage[k]['hit'] / coverage[k]['total'] if coverage[k]['total'] > 0 else 1.0)

    for filename in sorted_files:
        data = coverage[filename]
        if data['total'] == 0:
            continue
            
        percent = (data['hit'] / data['total']) * 100
        if percent == 100:
            continue
            
        print(f"\nFile: {filename}")
        print(f"Coverage: {percent:.2f}% ({data['hit']}/{data['total']})")
        
        missed_lines = [ln for ln, count in data['lines'].items() if count == 0]
        missed_lines.sort()
        
        # Group consecutive lines
        ranges = []
        if missed_lines:
            start = missed_lines[0]
            prev = start
            for ln in missed_lines[1:]:
                if ln > prev + 1:
                    if start == prev:
                        ranges.append(str(start))
                    else:
                        ranges.append(f"{start}-{prev}")
                    start = ln
                prev = ln
            if start == prev:
                ranges.append(str(start))
            else:
                ranges.append(f"{start}-{prev}")
                
        print(f"Missed Lines: {', '.join(ranges)}")

if __name__ == "__main__":
    lcov_path = "lcov.info"
    if len(sys.argv) > 1:
        lcov_path = sys.argv[1]
        
    cov = parse_lcov(lcov_path)
    if cov:
        print_gaps(cov)
