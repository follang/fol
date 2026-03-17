module.exports = grammar({
  name: 'fol',

  extras: $ => [
    /\s/,
    $.comment,
    $.doc_comment,
  ],

  conflicts: $ => [
    [$.qualified_path, $.call_expr],
    [$.record_literal, $.block],
    [$.record_literal, $.container_literal],
    [$.block, $.stmt],
    [$.block, $.record_literal, $.container_literal],
    [$.block, $.stmt, $.container_literal],
    [$.block, $.container_literal],
  ],

  rules: {
    source_file: $ => repeat($._top_level_item),

    _top_level_item: $ => choice(
      $.use_decl,
      $.var_decl,
      $.fun_decl,
      $.log_decl,
      $.typ_decl,
      $.ali_decl,
      $.comment,
      $.doc_comment,
    ),

    use_decl: $ => seq('use', field('name', $.identifier), ':', field('source_kind', $.source_kind), '=', '{', $.qualified_path, '}'),
    var_decl: $ => seq('var', $.typed_binding, '=', field('value', $.expr)),
    fun_decl: $ => seq('fun', optional(field('modifiers', $.decl_modifiers)), field('declaration', choice($.plain_fun_decl, $.method_decl))),
    log_decl: $ => seq('log', optional(field('modifiers', $.decl_modifiers)), field('declaration', choice($.plain_log_decl, $.method_decl))),
    typ_decl: $ => seq('typ', optional(field('modifiers', $.decl_modifiers)), field('name', $.identifier), ':', choice($.record_type, $.entry_type), '=', $.block),
    ali_decl: $ => seq('ali', optional(field('modifiers', $.decl_modifiers)), field('name', $.identifier), ':', field('target', $.type_expr)),

    source_kind: _ => choice('loc', 'std', 'pkg'),
    decl_modifiers: $ => seq('[', optional($.modifier_list), ']'),
    modifier_list: $ => seq($.identifier, repeat(seq(choice(',', ';'), $.identifier)), optional(choice(',', ';'))),
    typed_binding: $ => seq(field('name', $.identifier), ':', field('type', $.type_expr)),
    plain_fun_decl: $ => seq(field('name', $.identifier), $.params, optional($.error_type), '=', $.block),
    plain_log_decl: $ => seq(field('name', $.identifier), $.params, ':', 'bol', '=', $.block),
    method_decl: $ => seq($.receiver, field('name', $.identifier), $.params, optional($.error_type), '=', $.block),
    receiver: $ => seq('(', $.type_expr, ')'),
    params: $ => seq('(', optional(commaSep1($.param)), ')'),
    param: $ => seq(field('name', $.identifier), ':', field('type', $.type_expr)),
    error_type: $ => seq('/', $.type_expr),
    record_type: _ => 'rec',
    entry_type: _ => 'ent',

    type_expr: $ => choice(
      $.qualified_path,
      $.identifier,
      $.container_type,
      $.shell_type,
    ),

    container_type: $ => seq(choice('arr', 'vec', 'seq', 'set', 'map'), '[', commaSep1($.type_expr), ']'),
    shell_type: $ => seq(choice('opt', 'err'), '[', $.type_expr, ']'),

    block: $ => seq('{', repeat($.stmt), optional($.expr), '}'),
    stmt: $ => choice(
      $.var_decl,
      $.return_stmt,
      $.report_stmt,
      $.panic_stmt,
      $.unreachable_stmt,
      $.break_stmt,
      $.when_expr,
      $.loop_expr,
      $.expr,
    ),

    return_stmt: $ => prec.right(seq('return', optional($.expr))),
    report_stmt: $ => prec.right(seq('report', $.expr)),
    panic_stmt: $ => prec.right(seq('panic', $.expr)),
    unreachable_stmt: _ => 'unreachable',
    break_stmt: $ => prec.right(seq('break', optional($.expr))),
    when_expr: $ => seq('when', '(', $.expr, ')', $.block),
    loop_expr: $ => seq('loop', optional($.expr), $.block),

    expr: $ => choice(
      $.check_expr,
      $.call_expr,
      $.dot_intrinsic,
      $.qualified_path,
      $.identifier,
      $.record_literal,
      $.container_literal,
      $.string_literal,
      $.integer_literal,
      $.nil_literal,
      $.unwrap_expr,
    ),

    call_expr: $ => seq(field('callee', $.qualified_path), '(', optional(commaSep1($.expr)), ')'),
    check_expr: $ => seq('check', '(', $.expr, ')'),
    dot_intrinsic: $ => seq('.', field('name', $.identifier), '(', optional(commaSep1($.expr)), ')'),
    unwrap_expr: $ => seq(choice($.identifier, $.qualified_path), '!'),
    record_literal: $ => seq('{', optional(commaSep1($.field_init)), '}'),
    field_init: $ => seq(field('name', $.identifier), '=', field('value', $.expr)),
    container_literal: $ => seq('{', optional(commaSep1($.expr)), '}'),

    qualified_path: $ => seq(field('root', $.identifier), repeat1(seq('::', field('segment', $.identifier)))),
    identifier: _ => /[A-Za-z_][A-Za-z0-9_]*/,
    integer_literal: _ => /[0-9]+/,
    string_literal: _ => /"([^"\\]|\\.)*"/,
    nil_literal: _ => 'nil',
    comment: _ => token(choice(/`[^`\n]*`/, /\/\/[^\n]*/)),
    doc_comment: _ => token(/`\[[^`\n]*\][^`\n]*`/),
  }
});

function commaSep1(rule) {
  return seq(rule, repeat(seq(',', rule)));
}
