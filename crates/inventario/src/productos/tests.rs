use kernel::{Decimal, Uuid};

use super::*;

fn fields(nombre: &str) -> ProductoFields<'_> {
    ProductoFields {
        nombre,
        categoria_id: CategoriaId(Uuid::from_u128(1)),
        unidad_medida_id: UnidadMedidaId(Uuid::from_u128(2)),
        precio_venta: "",
        descripcion: "",
        marca: "",
        modelo: "",
        color: "",
    }
}

#[test]
fn los_textos_se_guardan_sin_espacios_en_las_orillas() {
    let producto = NewProducto::new(ProductoFields {
        marca: "  Bic ",
        modelo: " Cristal ",
        color: " Azul ",
        descripcion: "  Punto mediano  ",
        ..fields("  PLUMA AZUL  ")
    })
    .unwrap();

    assert_eq!(producto.nombre(), "PLUMA AZUL");
    assert_eq!(producto.marca(), Some("Bic"));
    assert_eq!(producto.modelo(), Some("Cristal"));
    assert_eq!(producto.color(), Some("Azul"));
    assert_eq!(producto.descripcion(), Some("Punto mediano"));
}

#[test]
fn el_nombre_se_guarda_como_se_escribio_sin_cambiar_mayusculas() {
    for nombre in ["PLUMA AZUL", "pluma azul", "Pluma IMSS"] {
        assert_eq!(NewProducto::new(fields(nombre)).unwrap().nombre(), nombre);
    }
}

#[test]
fn marca_modelo_color_y_descripcion_vacios_quedan_sin_valor() {
    let producto = NewProducto::new(ProductoFields {
        marca: "   ",
        ..fields("PLUMA")
    })
    .unwrap();

    assert_eq!(producto.marca(), None);
    assert_eq!(producto.modelo(), None);
    assert_eq!(producto.color(), None);
    assert_eq!(producto.descripcion(), None);
}

#[test]
fn un_nombre_vacio_o_solo_espacios_no_se_acepta() {
    assert_eq!(
        NewProducto::new(fields("")),
        Err(ProductoError::EmptyNombre)
    );
    assert_eq!(
        NewProducto::new(fields("   ")),
        Err(ProductoError::EmptyNombre)
    );
}

#[test]
fn un_nombre_de_150_caracteres_se_acepta_y_de_151_no() {
    assert!(NewProducto::new(fields(&"ñ".repeat(150))).is_ok());
    assert_eq!(
        NewProducto::new(fields(&"a".repeat(151))),
        Err(ProductoError::LongNombre)
    );
}

#[test]
fn sin_precio_el_producto_queda_por_llegar() {
    assert_eq!(
        NewProducto::new(fields("PLUMA")).unwrap().precio_venta(),
        None
    );
}

#[test]
fn con_precio_se_guarda_el_precio_validado() {
    let producto = NewProducto::new(ProductoFields {
        precio_venta: " 12.50 ",
        ..fields("PLUMA")
    })
    .unwrap();

    let precio = producto.precio_venta().map(|p| p.value());
    assert_eq!(precio, Some(Decimal::new(1250, 2)));
}

#[test]
fn un_precio_invalido_no_se_acepta_y_dice_por_que() {
    let result = NewProducto::new(ProductoFields {
        precio_venta: "12.345",
        ..fields("PLUMA")
    });

    assert_eq!(
        result,
        Err(ProductoError::Precio(PrecioError::TooManyDecimals))
    );
    assert_eq!(
        result.unwrap_err().to_string(),
        "El precio lleva como máximo dos decimales."
    );
}

/// Los datos de un producto con un texto puesto en uno de sus campos.
type WithText = fn(&str) -> ProductoFields<'_>;

#[test]
fn descripcion_marca_modelo_y_color_tienen_su_largo_maximo() {
    let casos: [(WithText, usize, ProductoError); 4] = [
        (
            |t| ProductoFields {
                descripcion: t,
                ..fields("PLUMA")
            },
            1000,
            ProductoError::LongDescripcion,
        ),
        (
            |t| ProductoFields {
                marca: t,
                ..fields("PLUMA")
            },
            150,
            ProductoError::LongMarca,
        ),
        (
            |t| ProductoFields {
                modelo: t,
                ..fields("PLUMA")
            },
            200,
            ProductoError::LongModelo,
        ),
        (
            |t| ProductoFields {
                color: t,
                ..fields("PLUMA")
            },
            150,
            ProductoError::LongColor,
        ),
    ];
    for (with, max, error) in casos {
        let justo = "ñ".repeat(max);
        let de_mas = "ñ".repeat(max + 1);
        assert!(
            NewProducto::new(with(&justo)).is_ok(),
            "{error:?} con {max}"
        );
        assert_eq!(NewProducto::new(with(&de_mas)), Err(error));
    }
}
