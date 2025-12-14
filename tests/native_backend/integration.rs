/*!
# Integration Tests for Native Backend

End-to-end tests that validate the complete compilation pipeline.
*/

#[cfg(feature = "llvm_backend")]
use super::utils::{NativeBackendTester, TestError, OptimizationLevel};

#[cfg(feature = "llvm_backend")]
#[cfg(test)]
mod tests {
    use super::*;

    fn get_tester() -> Result<NativeBackendTester, TestError> {
        let tester = NativeBackendTester::new()?;
        if !tester.is_backend_available() {
            eprintln!("Warning: LLVM backend not fully available, skipping integration tests");
            return Err(TestError::LLVMUnavailable);
        }
        Ok(tester)
    }

    #[test]
    fn test_complete_application() {
        let tester = match get_tester() {
            Ok(t) => t,
            Err(TestError::LLVMUnavailable) => return,
            Err(e) => panic!("Failed to create tester: {:?}", e),
        };

        // Complete application with multiple modules and features
        let vela_code = r#"
            // Data structures
            struct User {
                id: int,
                name: string,
                email: string,
                active: bool
            }

            struct Product {
                id: int,
                name: string,
                price: float,
                in_stock: bool
            }

            // Utility functions
            fn calculate_total(products: []Product) -> float {
                let total = 0.0;
                for i in 0..products.length() {
                    if products[i].in_stock {
                        total = total + products[i].price;
                    }
                }
                return total;
            }

            fn find_user_by_id(users: []User, id: int) -> User? {
                for i in 0..users.length() {
                    if users[i].id == id {
                        return users[i];
                    }
                }
                return none;
            }

            // Business logic
            fn process_order(user_id: int, product_ids: []int) -> string {
                // Mock user data
                let users = [
                    User { id: 1, name: "Alice", email: "alice@example.com", active: true },
                    User { id: 2, name: "Bob", email: "bob@example.com", active: true },
                    User { id: 3, name: "Charlie", email: "charlie@example.com", active: false }
                ];

                // Mock product data
                let products = [
                    Product { id: 1, name: "Laptop", price: 999.99, in_stock: true },
                    Product { id: 2, name: "Mouse", price: 29.99, in_stock: true },
                    Product { id: 3, name: "Keyboard", price: 79.99, in_stock: false }
                ];

                // Find user
                let user = find_user_by_id(users, user_id);
                if user == none {
                    return "User not found";
                }

                if !user.active {
                    return "User is inactive";
                }

                // Calculate order total
                let order_products = [];
                for i in 0..product_ids.length() {
                    let product_id = product_ids[i];
                    for j in 0..products.length() {
                        if products[j].id == product_id {
                            order_products.push(products[j]);
                            break;
                        }
                    }
                }

                let total = calculate_total(order_products);

                return "Order processed for ${user.name}: $${total}";
            }

            fn main() -> void {
                // Test the complete application
                let result1 = process_order(1, [1, 2]);  // Valid order
                let result2 = process_order(4, [1]);    // Invalid user
                let result3 = process_order(3, [1, 2]); // Inactive user

                print(result1);
                print(result2);
                print(result3);
            }
        "#;

        let result = tester.compile_and_run(vela_code, OptimizationLevel::Default)
            .expect("Complete application should compile and run");

        assert!(result.success, "Complete application should succeed");
        assert!(tester.validate_output("Order processed for Alice", &result.stdout));
        assert!(tester.validate_output("User not found", &result.stdout));
        assert!(tester.validate_output("User is inactive", &result.stdout));
    }

    #[test]
    fn test_file_io_operations() {
        let tester = match get_tester() {
            Ok(t) => t,
            Err(TestError::LLVMUnavailable) => return,
            Err(e) => panic!("Failed to create tester: {:?}", e),
        };

        // Test file I/O operations (if supported)
        let vela_code = r#"
            fn test_file_operations() -> void {
                // This would test file I/O if implemented
                // For now, just test basic string operations that might be used with files

                let data = "name,age,city\nAlice,30,New York\nBob,25,London";
                let lines = data.split("\n");

                print("Found ${lines.length()} lines");

                for i in 0..lines.length() {
                    let fields = lines[i].split(",");
                    if fields.length() >= 3 {
                        print("${fields[0]} is ${fields[1]} years old");
                    }
                }
            }

            fn main() -> void {
                test_file_operations();
            }
        "#;

        let result = tester.compile_and_run(vela_code, OptimizationLevel::Default)
            .expect("File operations test should succeed");

        assert!(result.success, "File operations simulation should succeed");
        assert!(tester.validate_output("Found 3 lines", &result.stdout));
        assert!(tester.validate_output("Alice is 30 years old", &result.stdout));
        assert!(tester.validate_output("Bob is 25 years old", &result.stdout));
    }

    #[test]
    fn test_network_simulation() {
        let tester = match get_tester() {
            Ok(t) => t,
            Err(TestError::LLVMUnavailable) => return,
            Err(e) => panic!("Failed to create tester: {:?}", e),
        };

        // Simulate network operations with data structures
        let vela_code = r#"
            struct HttpRequest {
                method: string,
                url: string,
                headers: map<string, string>,
                body: string
            }

            struct HttpResponse {
                status_code: int,
                headers: map<string, string>,
                body: string
            }

            fn simulate_api_call(request: HttpRequest) -> HttpResponse {
                if request.method == "GET" && request.url == "/users" {
                    return HttpResponse {
                        status_code: 200,
                        headers: { "Content-Type": "application/json" },
                        body: "[{\"id\":1,\"name\":\"Alice\"},{\"id\":2,\"name\":\"Bob\"}]"
                    };
                } else if request.method == "POST" && request.url == "/users" {
                    return HttpResponse {
                        status_code: 201,
                        headers: { "Content-Type": "application/json" },
                        body: "{\"id\":3,\"name\":\"Charlie\"}"
                    };
                } else {
                    return HttpResponse {
                        status_code: 404,
                        headers: {},
                        body: "Not Found"
                    };
                }
            }

            fn test_api_simulation() -> void {
                let get_request = HttpRequest {
                    method: "GET",
                    url: "/users",
                    headers: { "Accept": "application/json" },
                    body: ""
                };

                let post_request = HttpRequest {
                    method: "POST",
                    url: "/users",
                    headers: { "Content-Type": "application/json" },
                    body: "{\"name\":\"Charlie\"}"
                };

                let invalid_request = HttpRequest {
                    method: "GET",
                    url: "/invalid",
                    headers: {},
                    body: ""
                };

                let response1 = simulate_api_call(get_request);
                let response2 = simulate_api_call(post_request);
                let response3 = simulate_api_call(invalid_request);

                print("GET /users: ${response1.status_code}");
                print("POST /users: ${response2.status_code}");
                print("GET /invalid: ${response3.status_code}");
            }

            fn main() -> void {
                test_api_simulation();
            }
        "#;

        let result = tester.compile_and_run(vela_code, OptimizationLevel::Default)
            .expect("Network simulation should succeed");

        assert!(result.success, "Network simulation should succeed");
        assert!(tester.validate_output("GET /users: 200", &result.stdout));
        assert!(tester.validate_output("POST /users: 201", &result.stdout));
        assert!(tester.validate_output("GET /invalid: 404", &result.stdout));
    }

    #[test]
    fn test_database_simulation() {
        let tester = match get_tester() {
            Ok(t) => t,
            Err(TestError::LLVMUnavailable) => return,
            Err(e) => panic!("Failed to create tester: {:?}", e),
        };

        // Simulate database operations
        let vela_code = r#"
            struct DatabaseRecord {
                id: int,
                table: string,
                data: map<string, string>
            }

            fn simulate_query(table: string, condition: string) -> []DatabaseRecord {
                // Mock database simulation
                if table == "users" && condition == "active=true" {
                    return [
                        DatabaseRecord {
                            id: 1,
                            table: "users",
                            data: { "name": "Alice", "email": "alice@example.com", "active": "true" }
                        },
                        DatabaseRecord {
                            id: 2,
                            table: "users",
                            data: { "name": "Bob", "email": "bob@example.com", "active": "true" }
                        }
                    ];
                } else if table == "products" {
                    return [
                        DatabaseRecord {
                            id: 1,
                            table: "products",
                            data: { "name": "Laptop", "price": "999.99", "category": "electronics" }
                        }
                    ];
                }
                return [];
            }

            fn test_database_operations() -> void {
                let users = simulate_query("users", "active=true");
                let products = simulate_query("products", "");

                print("Found ${users.length()} active users");
                print("Found ${products.length()} products");

                for i in 0..users.length() {
                    print("User: ${users[i].data['name']} (${users[i].data['email']})");
                }

                for i in 0..products.length() {
                    print("Product: ${products[i].data['name']} - $${products[i].data['price']}");
                }
            }

            fn main() -> void {
                test_database_operations();
            }
        "#;

        let result = tester.compile_and_run(vela_code, OptimizationLevel::Default)
            .expect("Database simulation should succeed");

        assert!(result.success, "Database simulation should succeed");
        assert!(tester.validate_output("Found 2 active users", &result.stdout));
        assert!(tester.validate_output("Found 1 products", &result.stdout));
        assert!(tester.validate_output("User: Alice", &result.stdout));
        assert!(tester.validate_output("Product: Laptop", &result.stdout));
    }

    #[test]
    fn test_error_handling_integration() {
        let tester = match get_tester() {
            Ok(t) => t,
            Err(TestError::LLVMUnavailable) => return,
            Err(e) => panic!("Failed to create tester: {:?}", e),
        };

        // Test comprehensive error handling
        let vela_code = r#"
            enum Result<T, E> {
                Ok(T),
                Err(E)
            }

            struct AppError {
                code: int,
                message: string
            }

            fn validate_user(name: string, age: int) -> Result<string, AppError> {
                if name.length() == 0 {
                    return Err(AppError { code: 400, message: "Name cannot be empty" });
                }

                if age < 0 {
                    return Err(AppError { code: 400, message: "Age cannot be negative" });
                }

                if age < 18 {
                    return Err(AppError { code: 403, message: "User must be 18 or older" });
                }

                return Ok("User ${name} validated successfully");
            }

            fn process_user_registration(name: string, age: int) -> string {
                let validation = validate_user(name, age);

                match validation {
                    Ok(message) => {
                        return "Registration successful: ${message}";
                    }
                    Err(error) => {
                        return "Registration failed (${error.code}): ${error.message}";
                    }
                }
            }

            fn test_error_scenarios() -> void {
                let result1 = process_user_registration("Alice", 25);    // Success
                let result2 = process_user_registration("", 25);        // Empty name
                let result3 = process_user_registration("Bob", -5);     // Negative age
                let result4 = process_user_registration("Charlie", 16); // Too young

                print(result1);
                print(result2);
                print(result3);
                print(result4);
            }

            fn main() -> void {
                test_error_scenarios();
            }
        "#;

        let result = tester.compile_and_run(vela_code, OptimizationLevel::Default)
            .expect("Error handling integration should succeed");

        assert!(result.success, "Error handling should succeed");
        assert!(tester.validate_output("Registration successful", &result.stdout));
        assert!(tester.validate_output("Name cannot be empty", &result.stdout));
        assert!(tester.validate_output("Age cannot be negative", &result.stdout));
        assert!(tester.validate_output("User must be 18 or older", &result.stdout));
    }

    #[test]
    fn test_concurrent_processing_simulation() {
        let tester = match get_tester() {
            Ok(t) => t,
            Err(TestError::LLVMUnavailable) => return,
            Err(e) => panic!("Failed to create tester: {:?}", e),
        };

        // Simulate concurrent processing with data structures
        let vela_code = r#"
            struct Task {
                id: int,
                description: string,
                priority: int,
                completed: bool
            }

            struct Worker {
                id: int,
                tasks_processed: int,
                total_time: int
            }

            fn simulate_task_processing(tasks: []Task, num_workers: int) -> []Worker {
                let workers = [];
                for i in 0..num_workers {
                    workers.push(Worker { id: i, tasks_processed: 0, total_time: 0 });
                }

                // Simple round-robin task assignment simulation
                for i in 0..tasks.length() {
                    let worker_index = i % num_workers;
                    let task = tasks[i];

                    if !task.completed {
                        workers[worker_index].tasks_processed = workers[worker_index].tasks_processed + 1;
                        workers[worker_index].total_time = workers[worker_index].total_time + task.priority * 10;
                    }
                }

                return workers;
            }

            fn test_concurrent_simulation() -> void {
                let tasks = [
                    Task { id: 1, description: "Process payment", priority: 3, completed: false },
                    Task { id: 2, description: "Send email", priority: 1, completed: false },
                    Task { id: 3, description: "Generate report", priority: 2, completed: true },
                    Task { id: 4, description: "Update database", priority: 3, completed: false },
                    Task { id: 5, description: "Cache cleanup", priority: 1, completed: false }
                ];

                let workers = simulate_task_processing(tasks, 3);

                print("Workers performance:");
                for i in 0..workers.length() {
                    let worker = workers[i];
                    print("Worker ${worker.id}: ${worker.tasks_processed} tasks, ${worker.total_time} time units");
                }
            }

            fn main() -> void {
                test_concurrent_simulation();
            }
        "#;

        let result = tester.compile_and_run(vela_code, OptimizationLevel::Default)
            .expect("Concurrent processing simulation should succeed");

        assert!(result.success, "Concurrent simulation should succeed");
        assert!(tester.validate_output("Workers performance:", &result.stdout));
        assert!(tester.validate_output("tasks", &result.stdout));
        assert!(tester.validate_output("time units", &result.stdout));
    }

    #[test]
    fn test_full_stack_application() {
        let tester = match get_tester() {
            Ok(t) => t,
            Err(TestError::LLVMUnavailable) => return,
            Err(e) => panic!("Failed to create tester: {:?}", e),
        };

        // Simulate a full-stack application with multiple layers
        let vela_code = r#"
            // Domain Layer
            struct Order {
                id: int,
                customer_id: int,
                items: []OrderItem,
                total: float,
                status: string
            }

            struct OrderItem {
                product_id: int,
                quantity: int,
                unit_price: float
            }

            // Service Layer
            fn calculate_order_total(items: []OrderItem) -> float {
                let total = 0.0;
                for i in 0..items.length() {
                    total = total + (items[i].quantity * items[i].unit_price);
                }
                return total;
            }

            fn validate_order(order: Order) -> Result<Order, string> {
                if order.items.length() == 0 {
                    return Err("Order must have at least one item");
                }

                if order.customer_id <= 0 {
                    return Err("Invalid customer ID");
                }

                let calculated_total = calculate_order_total(order.items);
                if calculated_total != order.total {
                    return Err("Order total mismatch");
                }

                return Ok(order);
            }

            // Repository Layer (simulated)
            fn save_order(order: Order) -> Result<int, string> {
                // Simulate database save
                if order.id == 0 {
                    return Ok(12345); // New ID
                }
                return Ok(order.id);
            }

            // Controller Layer
            fn process_order_request(customer_id: int, items: []OrderItem) -> string {
                let order = Order {
                    id: 0,
                    customer_id: customer_id,
                    items: items,
                    total: calculate_order_total(items),
                    status: "pending"
                };

                let validation = validate_order(order);
                match validation {
                    Ok(valid_order) => {
                        let save_result = save_order(valid_order);
                        match save_result {
                            Ok(order_id) => {
                                return "Order ${order_id} created successfully for customer ${customer_id}";
                            }
                            Err(error) => {
                                return "Failed to save order: ${error}";
                            }
                        }
                    }
                    Err(error) => {
                        return "Order validation failed: ${error}";
                    }
                }
            }

            fn test_full_application() -> void {
                // Test successful order
                let items1 = [
                    OrderItem { product_id: 1, quantity: 2, unit_price: 10.0 },
                    OrderItem { product_id: 2, quantity: 1, unit_price: 25.0 }
                ];
                let result1 = process_order_request(123, items1);

                // Test empty order
                let result2 = process_order_request(123, []);

                // Test invalid customer
                let items3 = [OrderItem { product_id: 1, quantity: 1, unit_price: 10.0 }];
                let result3 = process_order_request(0, items3);

                print(result1);
                print(result2);
                print(result3);
            }

            fn main() -> void {
                test_full_application();
            }
        "#;

        let result = tester.compile_and_run(vela_code, OptimizationLevel::Default)
            .expect("Full stack application should succeed");

        assert!(result.success, "Full stack application should succeed");
        assert!(tester.validate_output("Order 12345 created successfully", &result.stdout));
        assert!(tester.validate_output("Order must have at least one item", &result.stdout));
        assert!(tester.validate_output("Invalid customer ID", &result.stdout));
    }
}