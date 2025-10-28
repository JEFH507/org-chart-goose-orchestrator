# Prompts

Local LLM prompt (NER assist):
- System: Detect sensitive entities (PII, secrets). Return spans and types only. Never output content.
- User: Text: {{text}}. Return JSON: {entities:[{type,start,end,score}]}
