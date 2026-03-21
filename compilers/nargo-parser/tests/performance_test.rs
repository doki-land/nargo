use nargo_parser::parse;
use std::time::Instant;

/// 性能测试
#[test]
pub fn test_parser_performance() {
    // 测试大型文件解析性能
    let large_file_content = generate_large_file(10000);

    // 测量解析时间
    let start = Instant::now();
    let result = parse(&large_file_content);
    let duration = start.elapsed();

    assert!(result.is_ok(), "解析失败: {:?}", result);
    println!("解析大型文件耗时: {:?}", duration);
    assert!(duration.as_millis() < 1000, "解析时间过长: {:?}", duration);
}

/// 生成大型测试文件
fn generate_large_file(size: usize) -> String {
    let mut content = String::with_capacity(size * 10);

    // 添加模板部分
    content.push_str(
        "<template>
",
    );
    for i in 0..size / 10 {
        content.push_str(&format!(
            "  <div class=\"item-{}”>
",
            i
        ));
        content.push_str(&format!(
            "    <h1>Item {}</h1>
",
            i
        ));
        content.push_str(&format!(
            "    <p>Content for item {}</p>
",
            i
        ));
        content.push_str(
            "  </div>
",
        );
    }
    content.push_str(
        "</template>

",
    );

    // 添加脚本部分
    content.push_str(
        "<script>
",
    );
    content.push_str(
        "import { ref, computed } from 'vue';

",
    );
    content.push_str(
        "export default {
",
    );
    content.push_str(
        "  name: 'LargeComponent',
",
    );
    content.push_str(
        "  setup() {
",
    );
    content.push_str("    const items = ref([");
    for i in 0..size / 100 {
        content.push_str(&format!("{{ id: {}, name: 'Item {}' }}, ", i, i));
    }
    content.push_str(
        "]);

",
    );
    content.push_str(
        "    const totalItems = computed(() => items.value.length);

",
    );
    content.push_str(
        "    function addItem() {
",
    );
    content.push_str(
        "      items.value.push({ id: items.value.length, name: 'New Item' });
",
    );
    content.push_str(
        "    }

",
    );
    content.push_str(
        "    return {
",
    );
    content.push_str(
        "      items,
",
    );
    content.push_str(
        "      totalItems,
",
    );
    content.push_str(
        "      addItem
",
    );
    content.push_str(
        "    };
",
    );
    content.push_str(
        "  }
",
    );
    content.push_str(
        "};
",
    );
    content.push_str(
        "</script>

",
    );

    // 添加样式部分
    content.push_str(
        "<style scoped>
",
    );
    for i in 0..size / 20 {
        content.push_str(&format!(
            ".item-{} {{
",
            i
        ));
        content.push_str(&format!(
            "  color: #{};
",
            i % 16777215
        ));
        content.push_str(&format!(
            "  font-size: {}px;
",
            12 + (i % 10)
        ));
        content.push_str(
            "}

",
        );
    }
    content.push_str("</style>");

    content
}

/// 测试 TypeScript 特性解析性能
#[test]
pub fn test_typescript_performance() {
    let ts_content = generate_typescript_content();

    let start = Instant::now();
    let result = parse(&ts_content);
    let duration = start.elapsed();

    assert!(result.is_ok(), "TypeScript 解析失败: {:?}", result);
    println!("TypeScript 解析耗时: {:?}", duration);
    assert!(duration.as_millis() < 500, "TypeScript 解析时间过长: {:?}", duration);
}

/// 生成 TypeScript 测试内容
fn generate_typescript_content() -> String {
    let mut content = String::new();

    content.push_str(
        "<script lang=\"ts\">
",
    );
    content.push_str(
        "import { ref, computed } from 'vue';

",
    );
    content.push_str(
        "interface User {
",
    );
    content.push_str(
        "  id: number;
",
    );
    content.push_str(
        "  name: string;
",
    );
    content.push_str(
        "  email: string;
",
    );
    content.push_str(
        "  age?: number;
",
    );
    content.push_str(
        "}

",
    );
    content.push_str(
        "type UserState = {
",
    );
    content.push_str(
        "  users: User[];
",
    );
    content.push_str(
        "  loading: boolean;
",
    );
    content.push_str(
        "  error: string | null;
",
    );
    content.push_str(
        "};

",
    );
    content.push_str(
        "type UserAction<T> = T extends 'add' ? { type: 'add'; payload: User } :
",
    );
    content.push_str(
        "T extends 'remove' ? { type: 'remove'; payload: number } :
",
    );
    content.push_str(
        "{ type: 'update'; payload: { id: number; user: Partial<User> } };

",
    );
    content.push_str(
        "export default {
",
    );
    content.push_str(
        "  name: 'TypeScriptComponent',
",
    );
    content.push_str(
        "  setup() {
",
    );
    content.push_str(
        "    const state = ref<UserState>({
",
    );
    content.push_str(
        "      users: [],
",
    );
    content.push_str(
        "      loading: false,
",
    );
    content.push_str(
        "      error: null
",
    );
    content.push_str(
        "    });

",
    );
    content.push_str(
        "    const totalUsers = computed(() => state.value.users.length);

",
    );
    content.push_str(
        "    function dispatch<T extends keyof UserAction<any>>(action: UserAction<T>) {
",
    );
    content.push_str(
        "      switch (action.type) {
",
    );
    content.push_str(
        "        case 'add':
",
    );
    content.push_str(
        "          state.value.users.push(action.payload);
",
    );
    content.push_str(
        "          break;
",
    );
    content.push_str(
        "        case 'remove':
",
    );
    content.push_str(
        "          state.value.users = state.value.users.filter(u => u.id !== action.payload);
",
    );
    content.push_str(
        "          break;
",
    );
    content.push_str(
        "        case 'update':
",
    );
    content.push_str(
        "          const index = state.value.users.findIndex(u => u.id === action.payload.id);
",
    );
    content.push_str(
        "          if (index !== -1) {
",
    );
    content.push_str(
        "            state.value.users[index] = { ...state.value.users[index], ...action.payload.user };
",
    );
    content.push_str(
        "          }
",
    );
    content.push_str(
        "          break;
",
    );
    content.push_str(
        "      }
",
    );
    content.push_str(
        "    }

",
    );
    content.push_str(
        "    return {
",
    );
    content.push_str(
        "      state,
",
    );
    content.push_str(
        "      totalUsers,
",
    );
    content.push_str(
        "      dispatch
",
    );
    content.push_str(
        "    };
",
    );
    content.push_str(
        "  }
",
    );
    content.push_str(
        "};
",
    );
    content.push_str("</script>");

    content
}
