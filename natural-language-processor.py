#!/usr/bin/env python3
"""
Claude Code Stacks - Natural Language Processor
Interprets user requests and routes to appropriate stack workflows
"""

import re
import subprocess
import sys
from pathlib import Path
import json
from typing import List, Dict, Optional

class StackNLProcessor:
    def __init__(self):
        self.project_root = Path.cwd()
        self.claude_dir = self.project_root / ".claude"
        self.stacks_dir = self.claude_dir / "stacks"
        
        # Load available stacks
        self.available_stacks = self._discover_stacks()
        
        # Intent patterns for different stack operations
        self.intent_patterns = {
            'linting': [
                r'\b(lint|linting|style|format|prettier|eslint|ruff|clippy)\b',
                r'\b(code style|fix.*style|format.*code)\b',
                r'\b(check.*code|validate.*syntax)\b'
            ],
            'testing': [
                r'\b(test|testing|spec|docker.*test|nginx.*test)\b',
                r'\b(run.*test|execute.*test|test.*suite)\b',
                r'\b(validate.*docker|check.*compose)\b'
            ],
            'clark_style': [
                r'\b(clark|style.*guide|remove.*emoji|concise)\b',
                r'\b(clean.*up|simplify|minimize)\b',
                r'\b(uv|bun|package.*manager)\b'
            ],
            'git': [
                r'\b(git|commit|push|pull|merge|branch)\b',
                r'\b(version.*control|repository|repo)\b',
                r'\b(subtree|contribute|deploy.*changes)\b'
            ],
            'cicd': [
                r'\b(ci|cd|deploy|pipeline|github.*action|gitlab)\b',
                r'\b(release|build|continuous)\b',
                r'\b(workflow|automation)\b'
            ],
            'docs': [
                r'\b(doc|documentation|readme|design.*doc)\b',
                r'\b(google.*drive|sync.*doc|generate.*doc)\b',
                r'\b(architecture|specification)\b'
            ],
            'database': [
                r'\b(database|db|postgres|mongo|redis|sql)\b',
                r'\b(connection|schema|migration)\b',
                r'\b(docker.*compose.*db|mcp.*config)\b'
            ]
        }
    
    def _discover_stacks(self) -> Dict[str, Dict]:
        """Discover available stacks in the project"""
        stacks = {}
        
        if not self.stacks_dir.exists():
            return stacks
            
        for stack_dir in self.stacks_dir.glob("stack-*"):
            if stack_dir.is_dir():
                stack_info = self._load_stack_info(stack_dir)
                stacks[stack_dir.name] = stack_info
                
        return stacks
    
    def _load_stack_info(self, stack_path: Path) -> Dict:
        """Load information about a specific stack"""
        claude_md = stack_path / "CLAUDE.md"
        info = {
            'path': stack_path,
            'description': 'No description available',
            'capabilities': []
        }
        
        if claude_md.exists():
            content = claude_md.read_text()
            # Extract description
            desc_match = re.search(r'^# Description: (.+)$', content, re.MULTILINE)
            if desc_match:
                info['description'] = desc_match.group(1)
                
        return info
    
    def analyze_intent(self, user_input: str) -> Dict[str, float]:
        """Analyze user input and determine intent scores for each stack type"""
        user_input_lower = user_input.lower()
        intent_scores = {}
        
        for intent, patterns in self.intent_patterns.items():
            score = 0.0
            for pattern in patterns:
                matches = len(re.findall(pattern, user_input_lower))
                score += matches * 1.0
            
            # Normalize score
            intent_scores[intent] = min(score, 5.0) / 5.0
            
        return intent_scores
    
    def route_request(self, user_input: str) -> List[str]:
        """Route user request to appropriate stacks"""
        intent_scores = self.analyze_intent(user_input)
        
        # Determine which stacks to activate based on intent scores
        active_stacks = []
        threshold = 0.2
        
        stack_mapping = {
            'linting': 'stack-1',
            'testing': 'stack-2', 
            'clark_style': 'stack-3',
            'git': 'stack-4',
            'cicd': 'stack-5',
            'docs': 'stack-6',
            'database': 'stack-7'
        }
        
        for intent, score in intent_scores.items():
            if score >= threshold and intent in stack_mapping:
                stack_name = stack_mapping[intent]
                if stack_name in self.available_stacks:
                    active_stacks.append(stack_name)
        
        # If no specific intent detected, use default stacks
        if not active_stacks:
            default_stacks = ['stack-1', 'stack-3']  # linting + style
            active_stacks = [s for s in default_stacks if s in self.available_stacks]
            
        return active_stacks
    
    def execute_claude_headless(self, prompt: str, mode: str = "auto-accept") -> str:
        """Execute Claude Code in headless mode with the given prompt"""
        try:
            cmd = ["claude", f"--mode={mode}", "-p", prompt]
            result = subprocess.run(cmd, capture_output=True, text=True, cwd=self.project_root)
            return result.stdout
        except subprocess.CalledProcessError as e:
            return f"Error executing Claude Code: {e}"
        except FileNotFoundError:
            return "Claude Code not found. Please ensure it's installed and in PATH."
    
    def process_request(self, user_input: str) -> str:
        """Main entry point for processing natural language requests"""
        print(f"🤖 Processing: {user_input}")
        
        # Analyze intent and route to stacks
        active_stacks = self.route_request(user_input)
        
        if not active_stacks:
            return "No relevant stacks found for this request. Run 'stacks' to configure your project."
        
        print(f"📋 Activating stacks: {', '.join(active_stacks)}")
        
        # Create context-aware prompt for Claude Code
        context_prompt = self._build_context_prompt(user_input, active_stacks)
        
        # Execute with Claude Code headless mode
        result = self.execute_claude_headless(context_prompt)
        
        return result
    
    def _build_context_prompt(self, user_input: str, active_stacks: List[str]) -> str:
        """Build a context-aware prompt for Claude Code execution"""
        stack_descriptions = []
        
        for stack_name in active_stacks:
            if stack_name in self.available_stacks:
                desc = self.available_stacks[stack_name]['description']
                stack_descriptions.append(f"- {stack_name}: {desc}")
        
        prompt = f"""
The user requested: "{user_input}"

Available stacks in this project:
{chr(10).join(stack_descriptions)}

Based on the user's request, please:
1. Understand what they want to accomplish
2. Use the appropriate stack capabilities to fulfill their request
3. Provide clear feedback about what actions were taken
4. If you need approval for any operations, ask the user

Execute the request using natural language understanding and the available stack tools.
"""
        
        return prompt.strip()

def main():
    if len(sys.argv) < 2:
        print("Usage: natural-language-processor.py '<user request>'")
        sys.exit(1)
    
    user_input = " ".join(sys.argv[1:])
    processor = StackNLProcessor()
    result = processor.process_request(user_input)
    print(result)

if __name__ == "__main__":
    main()