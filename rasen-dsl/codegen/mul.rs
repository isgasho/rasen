//! Mul trait implementation

use codegen::{
    defs::{Category, Node},
    math::construct_type,
    operations::match_values,
};
use proc_macro2::{Ident, Span, TokenStream};

fn impl_vector_times_scalar(
    result: Ident,
    size: u32,
    vector: TokenStream,
    scalar: TokenStream,
) -> TokenStream {
    let v_fields: Vec<_> = {
        (0..size)
            .map(|i| Ident::new(&format!("v_{}", i), Span::call_site()))
            .collect()
    };
    let res_fields: Vec<_> = {
        v_fields
            .iter()
            .map(|f| {
                quote! { #f * other }
            })
            .collect()
    };

    quote! {
        let #result( #( #v_fields ),* ) = #vector;
        let other = #scalar;
        #result( #( #res_fields ),* ).into()
    }
}

fn impl_vector_times_matrix(
    result: Ident,
    size: u32,
    vector: TokenStream,
    matrix: TokenStream,
) -> TokenStream {
    let v_fields = { (0..size).map(|i| Ident::new(&format!("v_{}", i), Span::call_site())) };
    let res_fields = {
        (0..size).map(|i| {
            let sum = {
                (0..size).map(|j| {
                    let f = Ident::new(&format!("v_{}", j), Span::call_site());
                    let index = ((i * size) + j) as usize;
                    quote! { #f * matrix[#index] }
                })
            };

            quote! { #( #sum )+* }
        })
    };

    quote! {
        let #result( #( #v_fields ),* ) = #vector;
        let matrix = #matrix;
        #result( #( #res_fields ),* ).into()
    }
}

pub fn impl_mul_variant(left_type: Node, right_type: Node) -> Option<TokenStream> {
    let left_res = left_type.result.clone();
    let right_res = right_type.result.clone();

    let (result, mul_impl) = match (
        left_res.category,
        left_res.ty,
        right_res.category,
        right_res.ty,
    ) {
        (_, "bool", _, _) | (_, _, _, "bool") => return None,

        (Category::SCALAR, _, Category::SCALAR, _)
            if !left_type.is_value() && !right_type.is_value() =>
        {
            return None
        }

        (lc, lt, rc, rt) if lc == rc && lt == rt && left_res.size == right_res.size => (
            left_res.name.clone(),
            match lc {
                Category::SCALAR => {
                    quote! {
                        (lhs * rhs).into()
                    }
                }
                Category::VECTOR => {
                    let result = left_res.name.clone();
                    let l_fields: Vec<_> = {
                        (0..left_res.size.unwrap())
                            .map(|i| Ident::new(&format!("l_{}", i), Span::call_site()))
                            .collect()
                    };
                    let r_fields: Vec<_> = {
                        (0..left_res.size.unwrap())
                            .map(|i| Ident::new(&format!("r_{}", i), Span::call_site()))
                            .collect()
                    };
                    let res_fields: Vec<_> = {
                        l_fields
                            .iter()
                            .zip(r_fields.iter())
                            .map(|(l_f, r_f)| {
                                quote! { #l_f * #r_f }
                            })
                            .collect()
                    };

                    quote! {
                        let #result( #( #l_fields ),* ) = lhs;
                        let #result( #( #r_fields ),* ) = rhs;
                        #result( #( #res_fields ),* ).into()
                    }
                }
                Category::MATRIX => {
                    let result = left_res.name.clone();
                    let size = left_res.size.unwrap() as usize;
                    let res_fields: Vec<_> = {
                        (0..size)
                            .flat_map(|i| {
                                (0..size).map(move |j| {
                                    let sum = {
                                        (0..size).map(|k| {
                                            let l = i * size + k;
                                            let r = k * size + j;
                                            quote! { left_mat[#l] * right_mat[#r] }
                                        })
                                    };

                                    quote! { #( #sum )+* }
                                })
                            })
                            .collect()
                    };

                    quote! {
                        let left_mat = lhs.0;
                        let right_mat = rhs.0;
                        #result( [ #( #res_fields, )* ] ).into()
                    }
                }
            },
        ),

        (Category::VECTOR, lt, rc, rt)
            if lt == rt && left_res.size.unwrap() == right_res.size.or(left_res.size).unwrap() =>
        {
            (
                left_res.name.clone(),
                match rc {
                    Category::VECTOR => unreachable!(),
                    Category::SCALAR => impl_vector_times_scalar(
                        left_res.name.clone(),
                        left_res.size.unwrap(),
                        quote! { lhs },
                        quote! { rhs },
                    ),
                    Category::MATRIX => impl_vector_times_matrix(
                        left_res.name.clone(),
                        left_res.size.unwrap(),
                        quote! { lhs },
                        quote! { rhs.0 },
                    ),
                },
            )
        }
        (lc, lt, Category::VECTOR, rt)
            if lt == rt && right_res.size.unwrap() == left_res.size.or(right_res.size).unwrap() =>
        {
            (
                right_res.name.clone(),
                match lc {
                    Category::VECTOR => unreachable!(),
                    Category::SCALAR => impl_vector_times_scalar(
                        right_res.name.clone(),
                        right_res.size.unwrap(),
                        quote! { rhs },
                        quote! { lhs },
                    ),
                    Category::MATRIX => impl_vector_times_matrix(
                        right_res.name.clone(),
                        right_res.size.unwrap(),
                        quote! { rhs },
                        quote! { lhs.0 },
                    ),
                },
            )
        }

        _ => return None,
    };

    let is_raw = !left_type.is_value() && !right_type.is_value();
    let left_type = construct_type(left_type);
    let right_type = construct_type(right_type);

    if is_raw {
        Some(quote! {
            impl Mul<#right_type> for #left_type {
                type Output = #result;

                #[inline]
                fn mul(self, rhs: #right_type) -> Self::Output {
                    let lhs = self;
                    #mul_impl
                }
            }
        })
    } else {
        let func_impl = match_values(
            &[
                Ident::new("lhs", Span::call_site()),
                Ident::new("rhs", Span::call_site()),
            ],
            &mul_impl,
            quote! {
                let index = graph.add_node(Node::Multiply);
                graph.add_edge(lhs, index, 0);
                graph.add_edge(rhs, index, 1);
                index
            },
        );

        Some(quote! {
            impl Mul<#right_type> for #left_type {
                type Output = Value<#result>;

                #[inline]
                fn mul(self, rhs: #right_type) -> Self::Output {
                    let lhs = self;
                    #func_impl
                }
            }
        })
    }
}
