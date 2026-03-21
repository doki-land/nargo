use nargo_compiler::Compiler;

// #[test]
// fn test_voc_if() {
//     let mut compiler = Compiler::new();
//     let source = r#"
// <template>
//   <if test={show}>
//     <div>Visible</div>
//   </if>
//   <if test={!show}>
//     <div>Hidden</div>
//   </if>
// </template>
// <script>
// const [show, setShow] = createSignal(true);
// </script>
// "#;
//     let res = compiler.compile("IfTest", source).unwrap();
//     println!("Generated JS for if:\n{}", res.code);
// }

// #[test]
// fn test_voc_for() {
//     let mut compiler = Compiler::new();
//     let source = r#"
// <template>
//   <ul>
//     <for item in={items}>
//       <li key={item.id}>{{ item.name }}</li>
//     </for>
//   </ul>
// </template>
// <script>
// const items = [{ id: 1, name: 'A' }, { id: 2, name: 'B' }];
// </script>
// "#;
//     let res = compiler.compile("ForTest", source).unwrap();
//     println!("Generated JS for for:\n{}", res.code);
// }
