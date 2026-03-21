use std::sync::Arc;

use nargo_parser::{Parser, ParserRegistry};

#[test]
fn test_parse_simple_component() {
    let source = r#"
<template>
  <div class="container">
    <h1>{{ title }}</h1>
  </div>
</template>

<script lang="ts">
import { ref } from 'vue';

const title = ref('Hello Nargo');
</script>

<style scoped>
.container {
  padding: 20px;
}
</style>
"#;

    let registry = Arc::new(ParserRegistry::new());
    let mut parser = Parser::new("test.nargo".to_string(), source, registry);

    let result = parser.parse_all();
    assert!(result.is_ok());

    let module = result.unwrap();
    assert!(module.template.is_some());
    assert!(module.script.is_some());
    assert_eq!(module.styles.len(), 1);
}

#[test]
fn test_parse_template_only() {
    let source = r#"
<template>
  <div>Hello World</div>
</template>
"#;

    let registry = Arc::new(ParserRegistry::new());
    let mut parser = Parser::new("test.nargo".to_string(), source, registry);

    let result = parser.parse_all();
    assert!(result.is_ok());

    let module = result.unwrap();
    assert!(module.template.is_some());
    assert!(module.script.is_none());
    assert!(module.styles.is_empty());
}

#[test]
fn test_parse_script_only() {
    let source = r#"
<script lang="ts">
const message = 'Hello';
function greet() {
  return message;
}
</script>
"#;

    let registry = Arc::new(ParserRegistry::new());
    let mut parser = Parser::new("test.nargo".to_string(), source, registry);

    let result = parser.parse_all();
    assert!(result.is_ok());

    let module = result.unwrap();
    assert!(module.template.is_none());
    assert!(module.script.is_some());
}

#[test]
fn test_parse_vue_template_features() {
    let source = r#"
<template>
  <div class="container">
    <h1 v-if="showTitle">{{ title }}</h1>
    <button @click="toggleTitle">Toggle Title</button>
    <input v-model="message" placeholder="Enter message" />
    <slot name="header"></slot>
    <div v-for="item in items" :key="item.id">{{ item.name }}</div>
  </div>
</template>

<script lang="ts">
import { ref, reactive } from 'vue';

const showTitle = ref(true);
const title = ref('Hello Nargo');
const message = ref('');
const items = reactive([
  { id: 1, name: 'Item 1' },
  { id: 2, name: 'Item 2' }
]);

function toggleTitle() {
  showTitle.value = !showTitle.value;
}
</script>
"#;

    let registry = Arc::new(ParserRegistry::new());
    let mut parser = Parser::new("test.nargo".to_string(), source, registry);

    let result = parser.parse_all();
    assert!(result.is_ok());

    let module = result.unwrap();
    assert!(module.template.is_some());
    assert!(module.script.is_some());
}

#[test]
fn test_parse_typescript_features() {
    let source = r#"
<script lang="ts">
interface User {
  id: number;
  name: string;
  email: string;
}

class UserService {
  private users: User[] = [];

  addUser(user: User): void {
    this.users.push(user);
  }

  getUserById(id: number): User | undefined {
    return this.users.find(user => user.id === id);
  }
}

const userService = new UserService();
userService.addUser({ id: 1, name: 'John', email: 'john@example.com' });

const getUser = (id: number): string => {
  const user = userService.getUserById(id);
  return user ? user.name : 'User not found';
};
</script>
"#;

    let registry = Arc::new(ParserRegistry::new());
    let mut parser = Parser::new("test.nargo".to_string(), source, registry);

    let result = parser.parse_all();
    assert!(result.is_ok());

    let module = result.unwrap();
    assert!(module.script.is_some());
}

#[test]
fn test_parse_tailwind_styles() {
    let source = r#"
<template>
  <div class="bg-white p-4 rounded-lg shadow-md">
    <h1 class="text-2xl font-bold text-gray-800">Tailwind Example</h1>
    <p class="text-gray-600 mt-2">This is a Tailwind CSS example</p>
  </div>
</template>

<style scoped>
@tailwind base;
@tailwind components;
@tailwind utilities;

.custom-class {
  @apply bg-blue-500 text-white px-4 py-2 rounded;
}
</style>
"#;

    let registry = Arc::new(ParserRegistry::new());
    let mut parser = Parser::new("test.nargo".to_string(), source, registry);

    let result = parser.parse_all();
    assert!(result.is_ok());

    let module = result.unwrap();
    assert!(module.template.is_some());
    assert_eq!(module.styles.len(), 1);
}
