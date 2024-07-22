use oxc_ast::{
	ast::{ClassElement, Expression, MethodDefinition,
		MethodDefinitionKind, Statement, TSAccessibility},
	AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Default, Clone)]
pub struct ClassLiteralPropertyStyle {
	style: Style,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// ```
    ClassLiteralPropertyStyle,
    style, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details
);

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
enum Style {
    #[default]
    Fields,
    Getters,
}


fn prefer_field_style_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Literals should be exposed using readonly fields.").with_label(span)
}

fn get_is_supported_literal(expression: &Expression) -> bool {
	if expression.is_literal() {
		return true
	}
	match expression {
		Expression::TaggedTemplateExpression(tagged) => {
			tagged.quasi.quasis.len() == 1
		}
		Expression::TemplateLiteral(template_literal) => {
			template_literal.quasis.len() == 1
		}
		_ => {false}
	}
}

fn get_method_definition_modifiers(def: &MethodDefinition) -> String {
	let access_modifier = match def.accessibility {
		Some(TSAccessibility::Private) => "private",
		Some(TSAccessibility::Protected) => "protected",
		Some(TSAccessibility::Public) => "public",
		None => ""
	};
	let static_modifier = if def.r#static {" static"} else {""};
	format!("{}{}", access_modifier, static_modifier).to_string()
}

impl Rule for ClassLiteralPropertyStyle {
	fn from_configuration(value: serde_json::Value) -> Self {
        let style = value.get(0).and_then(serde_json::Value::as_str).map_or_else(
            Style::default,
            |value| match value {
                "getters" => Style::Getters,
                _ => Style::Fields,
            },
        );
        Self { style: style }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
		match self.style {
			Style::Getters => {
				AstKind
				let AstKind::ClassBody()
			}
			Style::Fields => {
				let AstKind::MethodDefinition(method_definition) = node.kind() else {return;};
				let MethodDefinitionKind::Get = method_definition.kind else {return;};
				let Some(ref body) = method_definition.value.body else {return;};
				let Some(statement) = body.statements.first() else {return;};
				let Statement::ReturnStatement(return_statement) = statement else {return;};
				let Some(ref argument) = return_statement.argument else {return;};
				if !get_is_supported_literal(argument) {return;};
				let name = method_definition.key.name();
				if let Some(parent) = ctx.nodes().parent_node(node.id()) {
					if let AstKind::ClassBody(class_body) = parent.kind() {
						let has_duplicate_key_setter = class_body.body.iter().any(|element| {
							let Some(MethodDefinitionKind::Set) = element.method_definition_kind() else {return false};
							name == method_definition.key.name()
						});
						if has_duplicate_key_setter {
							return
						}
					}
				}
				ctx.diagnostic(
					prefer_field_style_diagnostic(method_definition.span)
					// |fixer| {
					// 	let Some(name) = method_definition.key.name() else {return;};
					// 	let new_name = if method_definition.computed {format!("[{}]", name)} else {name};
					// 	let modifiers = get_method_definition_modifiers(method_definition);
					// 	let assignment = format!(" = {};", argument.)
					// 	let replace_str = format!("{}{}{}", modifiers, new_name, assignment);
					// 	fixer.replace(method_definition.span(), replace_str);
				)
			}
		}
	}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
			class Mx {
			  declare readonly p1 = 1;
			}
			    ",
            None,
        ),
        (
            "
			class Mx {
			  readonly p1 = 'hello world';
			}
			    ",
            None,
        ),
        (
            "
			class Mx {
			  p1 = 'hello world';
			}
			    ",
            None,
        ),
        (
            "
			class Mx {
			  static p1 = 'hello world';
			}
			    ",
            None,
        ),
        (
            "
			class Mx {
			  p1: string;
			}
			    ",
            None,
        ),
        (
            "
			class Mx {
			  get p1();
			}
			    ",
            None,
        ),
        (
            "
			class Mx {
			  get p1() {}
			}
			    ",
            None,
        ),
        (
            "
			abstract class Mx {
			  abstract get p1(): string;
			}
			    ",
            None,
        ),
        (
            "
			      class Mx {
			        get mySetting() {
			          if (this._aValue) {
			            return 'on';
			          }
			
			          return 'off';
			        }
			      }
			    ",
            None,
        ),
        (
            "
			      class Mx {
			        get mySetting() {
			          return `build-${process.env.build}`;
			        }
			      }
			    ",
            None,
        ),
        (
            "
			      class Mx {
			        getMySetting() {
			          if (this._aValue) {
			            return 'on';
			          }
			
			          return 'off';
			        }
			      }
			    ",
            None,
        ),
        (
            "
			      class Mx {
			        public readonly myButton = styled.button`
			          color: ${props => (props.primary ? 'hotpink' : 'turquoise')};
			        `;
			      }
			    ",
            None,
        ),
        (
            "
			      class Mx {
			        set p1(val) {}
			        get p1() {
			          return '';
			        }
			      }
			    ",
            None,
        ),
        (
            "
			      let p1 = 'p1';
			      class Mx {
			        set [p1](val) {}
			        get [p1]() {
			          return '';
			        }
			      }
			    ",
            None,
        ),
        (
            "
			      let p1 = 'p1';
			      class Mx {
			        set [/* before set */ p1 /* after set */](val) {}
			        get [/* before get */ p1 /* after get */]() {
			          return '';
			        }
			      }
			    ",
            None,
        ),
        (
            "
			      class Mx {
			        set ['foo'](val) {}
			        get foo() {
			          return '';
			        }
			        set bar(val) {}
			        get ['bar']() {
			          return '';
			        }
			        set ['baz'](val) {}
			        get baz() {
			          return '';
			        }
			      }
			    ",
            None,
        ),
        (
            "
			        class Mx {
			          public get myButton() {
			            return styled.button`
			              color: ${props => (props.primary ? 'hotpink' : 'turquoise')};
			            `;
			          }
			        }
			      ",
            Some(serde_json::json!(["fields"])),
        ),
        (
            "
			class Mx {
			  public declare readonly foo = 1;
			}
			      ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
			class Mx {
			  get p1() {
			    return 'hello world';
			  }
			}
			      ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
			class Mx {
			  p1 = 'hello world';
			}
			      ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
			class Mx {
			  p1: string;
			}
			      ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
			class Mx {
			  readonly p1 = [1, 2, 3];
			}
			      ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
			class Mx {
			  static p1: string;
			}
			      ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
			class Mx {
			  static get p1() {
			    return 'hello world';
			  }
			}
			      ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
			        class Mx {
			          public readonly myButton = styled.button`
			            color: ${props => (props.primary ? 'hotpink' : 'turquoise')};
			          `;
			        }
			      ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
			        class Mx {
			          public get myButton() {
			            return styled.button`
			              color: ${props => (props.primary ? 'hotpink' : 'turquoise')};
			            `;
			          }
			        }
			      ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
			        class A {
			          private readonly foo: string = 'bar';
			          constructor(foo: string) {
			            this.foo = foo;
			          }
			        }
			      ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
			        class A {
			          private readonly foo: string = 'bar';
			          constructor(foo: string) {
			            this['foo'] = foo;
			          }
			        }
			      ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
			        class A {
			          private readonly foo: string = 'bar';
			          constructor(foo: string) {
			            const bar = new (class {
			              private readonly foo: string = 'baz';
			              constructor() {
			                this.foo = 'qux';
			              }
			            })();
			            this['foo'] = foo;
			          }
			        }
			      ",
            Some(serde_json::json!(["getters"])),
        ),
    ];

    let fail = vec![
        (
            "
			class Mx {
			  get p1() {
			    return 'hello world';
			  }
			}
			      ",
            None,
        ),
        (
            "
			class Mx {
			  get p1() {
			    return `hello world`;
			  }
			}
			      ",
            None,
        ),
        (
            "
			class Mx {
			  static get p1() {
			    return 'hello world';
			  }
			}
			      ",
            None,
        ),
        (
            "
			class Mx {
			  public static get foo() {
			    return 1;
			  }
			}
			      ",
            None,
        ),
        (
            "
			class Mx {
			  public get [myValue]() {
			    return 'a literal value';
			  }
			}
			      ",
            None,
        ),
        (
            "
			class Mx {
			  public get [myValue]() {
			    return 12345n;
			  }
			}
			      ",
            None,
        ),
        (
            "
			class Mx {
			  public readonly [myValue] = 'a literal value';
			}
			      ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
			class Mx {
			  readonly p1 = 'hello world';
			}
			      ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
			class Mx {
			  readonly p1 = `hello world`;
			}
			      ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
			class Mx {
			  static readonly p1 = 'hello world';
			}
			      ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
			class Mx {
			  protected get p1() {
			    return 'hello world';
			  }
			}
			      ",
            Some(serde_json::json!(["fields"])),
        ),
        (
            "
			class Mx {
			  protected readonly p1 = 'hello world';
			}
			      ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
			class Mx {
			  public static get p1() {
			    return 'hello world';
			  }
			}
			      ",
            None,
        ),
        (
            "
			class Mx {
			  public static readonly p1 = 'hello world';
			}
			      ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
			class Mx {
			  public get myValue() {
			    return gql`
			      {
			        user(id: 5) {
			          firstName
			          lastName
			        }
			      }
			    `;
			  }
			}
			      ",
            None,
        ),
        (
            "
			class Mx {
			  public readonly myValue = gql`
			    {
			      user(id: 5) {
			        firstName
			        lastName
			      }
			    }
			  `;
			}
			      ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
			class A {
			  private readonly foo: string = 'bar';
			  constructor(foo: string) {
			    const bar = new (class {
			      private readonly foo: string = 'baz';
			      constructor() {
			        this.foo = 'qux';
			      }
			    })();
			  }
			}
			      ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
			class A {
			  private readonly ['foo']: string = 'bar';
			  constructor(foo: string) {
			    const bar = new (class {
			      private readonly foo: string = 'baz';
			      constructor() {}
			    })();
			
			    if (bar) {
			      this.foo = 'baz';
			    }
			  }
			}
			      ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
			class A {
			  private readonly foo: string = 'bar';
			  constructor(foo: string) {
			    function func() {
			      this.foo = 'aa';
			    }
			  }
			}
			      ",
            Some(serde_json::json!(["getters"])),
        ),
    ];

    Tester::new(ClassLiteralPropertyStyle::NAME, pass, fail).test_and_snapshot();
}
