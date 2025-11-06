#!/usr/bin/env python3
"""
Generate SQL seed data from YAML profile files.
Converts analyst.yaml and legal.yaml to INSERT statements for profiles table.
"""

import yaml
import json
import sys
from pathlib import Path

def yaml_to_sql_insert(yaml_file: Path) -> str:
    """Convert a YAML profile to SQL INSERT statement."""
    with open(yaml_file, 'r') as f:
        profile = yaml.safe_load(f)
    
    role = profile['role']
    display_name = profile['display_name']
    
    # Convert YAML to JSON for database storage
    data_json = json.dumps(profile, indent=2)
    
    # Escape single quotes for SQL
    data_json = data_json.replace("'", "''")
    
    sql = f"""
-- {display_name} Profile
INSERT INTO profiles (role, display_name, data, signature)
VALUES (
  '{role}',
  '{display_name}',
  '{data_json}'::jsonb,
  NULL
);
"""
    return sql

def main():
    profiles_dir = Path(__file__).parent.parent / 'profiles'
    
    # Generate for analyst and legal
    for profile_name in ['analyst', 'legal']:
        yaml_file = profiles_dir / f'{profile_name}.yaml'
        if not yaml_file.exists():
            print(f"Error: {yaml_file} not found", file=sys.stderr)
            sys.exit(1)
        
        sql = yaml_to_sql_insert(yaml_file)
        print(sql)

if __name__ == '__main__':
    main()
