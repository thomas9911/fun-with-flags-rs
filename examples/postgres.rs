extern crate diesel;

#[allow(unused_imports)]
use diesel::prelude::*;
#[allow(unused_imports)]
use fun_with_flags::establish_connection;
#[allow(unused_imports)]
use fun_with_flags::models::FeatureFlag;
fn main() {
    #[allow(unused_imports)]
    use fun_with_flags::schema::fun_with_flags_toggles::dsl::*;

    // let connection = establish_connection();
    // let results = fun_with_flags_toggles
    //     // .filter(enabled.eq(true))
    //     // .filter(flag_name.eq("boolean_one"))
    //     // .limit(5)
    //     .load::<FeatureFlag>(&connection)
    //     .expect("Error loading posts");

    // println!("Displaying {} feature flags", results.len());
    // for post in results {
    //     println!("{:?}", post);
    //     // println!("----------\n");
    //     // println!("{}", post.body);
    // }

    // fun_with_flags::enable("Hi").unwrap();
    // fun_with_flags::disable("Hi").unwrap();

    // let flag = FeatureFlag::Boolean {
    //     name: "Hi".to_string(),
    //     enabled: true,
    // };

    // let a = diesel::insert_into(fun_with_flags_toggles)
    //     .values(flag.to_insertable())
    //     .on_conflict((flag_name, gate_type, target))
    //     .do_update()
    //     .set(enabled.eq(flag.enabled()))
    //     .get_results::<FeatureFlag>(&connection)
    //     .unwrap();

    // println!("{}", diesel::debug_query::<diesel::pg::Pg, _>(&a));

    // println!("{:?}", a);
    // const FLAG_NAME: &str = "boolean_one";

    // if fun_with_flags::enabled(FLAG_NAME) {
    //     println!("active");
    //     fun_with_flags::disable(FLAG_NAME).unwrap();
    // } else {
    //     println!("not active");
    //     fun_with_flags::enable(FLAG_NAME).unwrap();
    // }

    // let x = fun_with_flags::enable_percentage("rust-percentage", 0.6).unwrap();
    let x = fun_with_flags::disable_percentage_of_actors("rust-percentage").unwrap();
    println!("{:?}", x);
}
