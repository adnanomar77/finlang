FinLang Language Reference



Language version: finlang-0.1



FinLang is a deterministic financial domain-specific programming language (DSL) designed for statically checked financial logic and controlled execution through a verified bytecode virtual machine.



This document describes the language syntax and the constructs implemented by the current FinLang compiler.



⸻



1\. Source Files



FinLang source files use the .fin extension.



Example:



program.fin



A source file contains a single FinLang expression.



A trailing semicolon is accepted.



mint(alice, 100);



Comments begin with // and continue to the end of the line.



// This is a comment.



⸻



2\. Literals



Integer



Integer literals are unsigned integers:



0

1

100

1000000



Decimal



Decimal literals are supported:



1.5

2.0

10.25



Decimals are currently used where the language expects a ratio, such as a loan collateral ratio.



Boolean



FinLang supports:



true

false



⸻



3\. Variables



Variables are introduced using let ... in ....



Syntax:



let name = expression in expression



Example:



let price = 100 in price



Variables can be used inside subsequent expressions.



Example:



let x = 100 in x + 50



⸻



4\. Conditional Expressions



FinLang supports conditional expressions:



if condition then expression else expression



Example:



if true then 100 else 0



The condition must be a Boolean expression.



A practical arithmetic example:



if (2 + 3 \* 4) >= 14 then 100 / 4 else 0



⸻



5\. Arithmetic Operators



FinLang supports:



Operator	Meaning

\+	Addition

\-	Subtraction

\*	Multiplication

/	Division



Example:



2 + 3 \* 4



Multiplication and division have higher precedence than addition and subtraction.



Therefore:



2 + 3 \* 4



is evaluated as:



2 + (3 \* 4)



Parentheses can be used to control evaluation:



(2 + 3) \* 4



⸻



6\. Comparison Operators



FinLang supports:



<

>

<=

>=

==



Examples:



100 >= 50

100 < 200

100 == 100



Comparison expressions produce Boolean results and can be used by conditional expressions and policies.



⸻



7\. Functions



Functions are defined using:



fn(parameters) -> ReturnType {

&#x20;   expression

}



Example:



fn(x: Amount, y: Amount) -> Amount {

&#x20;   x + y

}



A function can be assigned with let:



let add = fn(x: Amount, y: Amount) -> Amount {

&#x20;   x + y

} in add(100, 50)



Function calls use:



function(argument1, argument2)



Example:



add(100, 50)



⸻



8\. Built-in Financial Operations



FinLang provides language-level financial operations for manipulating the execution state.



The current language includes:



mint(...)

transfer(...)

createLoan(...)

repay(...)

priceUpdate(...)

liquidate(...)



These operations are checked by the compiler and enforced by the runtime.



⸻



9\. Mint



mint creates an amount for an account.



Syntax:



mint(account, amount)



Example:



mint(alice, 100)



The amount is added to the specified account using checked arithmetic.



⸻



10\. Transfer



transfer changes ownership between two accounts.



Syntax:



transfer(from, to, amount)



Example:



transfer(alice, bob, 50)



The runtime verifies that the transferred amount exists and is owned by the source account.



⸻



11\. Oracle Reads



FinLang supports explicit oracle inputs.



Syntax:



oracleRead(feedA)



or:



oracleRead(feedB)



The currently defined oracle sources are:



feedA

feedB



An oracle read has the type:



Oracle(USD, source)



Oracle values are explicit inputs to the execution environment.



They are therefore deterministic when the same oracle input sequence is supplied.



⸻



12\. Validation



An oracle value can be validated using a policy.



Syntax:



validate(oracleExpression, policy)



Example:



validate(

&#x20;   oracleRead(feedA),

&#x20;   PriceBounds

)



Validation converts an untrusted oracle value into a policy-bound verified value.



Conceptually:



Oracle

&#x20; ↓

validate(...)

&#x20; ↓

Verified



⸻



13\. PriceBounds



PriceBounds is the built-in policy identifier used by the verified oracle flow.



Example:



validate(

&#x20;   oracleRead(feedA),

&#x20;   PriceBounds

)



⸻



14\. Converting a Verified Value



toAmount converts a compatible verified value into an amount.



Syntax:



toAmount(verifiedExpression)



Example:



let price = toAmount(

&#x20;   validate(

&#x20;       oracleRead(feedA),

&#x20;       PriceBounds

&#x20;   )

) in

mint(alice, price)



This is the verified oracle flow demonstrated by the project’s loan.fin example.



⸻



15\. Complete Oracle Example



let price = toAmount(

&#x20;   validate(

&#x20;       oracleRead(feedA),

&#x20;       PriceBounds

&#x20;   )

) in

mint(alice, price)



The execution flow is:



oracleRead(feedA)

&#x20;       ↓

Oracle value

&#x20;       ↓

validate(..., PriceBounds)

&#x20;       ↓

Verified value

&#x20;       ↓

toAmount(...)

&#x20;       ↓

Amount

&#x20;       ↓

mint(alice, ...)



⸻



16\. Unsafe Trusted Conversion



FinLang also exposes:



unsafeAssumeTrusted(...)



Syntax:



unsafeAssumeTrusted(oracleExpression)



This operation explicitly bypasses the normal verified-oracle conversion path.



Because it is named unsafeAssumeTrusted, programs should treat it as a distinct trust boundary.



⸻



17\. Policies



FinLang supports user-defined executable policies.



Syntax:



policy Name(parameter: Type) {

&#x20;   predicate

}

in expression



Example:



policy Minimum(x: Amount) {

&#x20;   x >= 100

}

in validate(oracleRead(feedA), Minimum)



A policy consists of:



1\. A policy name

2\. A parameter

3\. An optional parameter type

4\. A predicate

5\. An expression executed after the policy definition



The predicate determines whether the policy accepts the value.



⸻



18\. Policy Rejection



For example:



policy Minimum(x: Amount) {

&#x20;   x >= 200

}

in validate(oracleRead(feedA), Minimum)



If the oracle value is 150, the VM rejects the execution because:



150 >= 200



is false.



The runtime reports:



policy 'Minimum' rejected value 150



⸻



19\. Loans



FinLang provides explicit loan lifecycle operations.



Create Loan



Syntax:



createLoan(

&#x20;   borrower,

&#x20;   lenderPool,

&#x20;   loanId,

&#x20;   amount,

&#x20;   collateralAsset,

&#x20;   collateralValue,

&#x20;   requiredRatio

)



Example:



createLoan(

&#x20;   alice,

&#x20;   pool,

&#x20;   loan1,

&#x20;   100,

&#x20;   collateral,

&#x20;   200,

&#x20;   1.5

)



The runtime records the loan, debt, receivable, collateral, and required collateral ratio.



The lender pool is debited and the borrower receives the loan amount.



⸻



20\. Repayment



Syntax:



repay(

&#x20;   borrower,

&#x20;   lenderPool,

&#x20;   loan,

&#x20;   payment

)



Example:



repay(alice, pool, loan1, 50)



Repayment transfers payment from the borrower to the lender pool.



A loan closes when its remaining debt reaches zero.



⸻



21\. Price Updates



Syntax:



priceUpdate(loan, newPrice)



Example:



priceUpdate(loan1, 180)



The operation recomputes the relevant loan status using the updated price.



⸻



22\. Liquidation



Syntax:



liquidate(loan)



Example:



liquidate(loan1)



Liquidation is permitted only when the loan is in a liquidatable state.



The runtime closes the debt and distributes the covered debt and surplus according to the financial execution rules.



⸻



23\. Types



The current FinLang type system contains the following types:



Amount

LinearAsset

Account

Oracle

Verified

Unit

Bool

Function

Loan

Debt

Collateral



Amount



Represents a financial amount associated with a currency.



The current implementation supports:



USD

EUR



Account



Represents an account identifier.



Oracle



Represents an oracle-derived value associated with a currency and source.



Verified



Represents an oracle value that has passed a specified policy.



Bool



Boolean values:



true

false



Function



Represents a function with parameter types and a result type.



Loan



Represents a loan identified by a loan ID.



Debt



Represents debt associated with a loan and currency.



Collateral



Represents collateral associated with a loan and currency.



LinearAsset



Represents a linear financial asset.



The current asset kind is:



Money(Currency)



⸻



24\. Supported Currencies



The current implementation defines:



USD

EUR



⸻



25\. Supported Oracle Sources



The current implementation defines:



feedA

feedB



⸻



26\. Linear Resources



FinLang uses a linear resource context for resources such as loans and assets.



A linear resource cannot be consumed more than once.



Conceptually:



resource

&#x20;  ↓

use

&#x20;  ↓

consumed



A second use of the same linear resource is rejected by the type system.



This prevents accidental repeated consumption of resources such as loans and linear assets.



⸻



27\. Deterministic Execution



FinLang execution is designed to be deterministic.



Given the same:



\* source program

\* compiled bytecode

\* initial financial state

\* oracle input sequence



the VM is expected to produce the same execution result and final state.



Oracle values are supplied explicitly as FIFO inputs.



State digests use deterministic key ordering before hashing.



⸻



28\. Atomic Execution



Financial execution uses transaction-style semantics.



The VM evaluates execution against a cloned state.



Conceptually:



Initial State

&#x20;     ↓

Clone

&#x20;     ↓

Execute

&#x20;     ↓

Success ──→ Commit

&#x20;     │

&#x20;     └────→ Failure → Discard



If execution fails, the original state is not committed.



This provides atomic execution semantics for the current runtime.



⸻



29\. Checked Arithmetic



Financial state updates use checked arithmetic.



Operations that would violate the supported arithmetic constraints are rejected rather than silently wrapping.



This applies particularly to financial balance updates and state transitions.



⸻



30\. Complete Example



Arithmetic



// Deterministic arithmetic and conditional example.

if (2 + 3 \* 4) >= 14 then 100 / 4 else 0



Verified Oracle Flow



// Verified oracle example.

let price = toAmount(

&#x20;   validate(

&#x20;       oracleRead(feedA),

&#x20;       PriceBounds

&#x20;   )

) in

mint(alice, price)



These examples are included in the repository under:



examples/arithmetic.fin

examples/loan.fin



⸻



31\. Command-Line Usage



After installation:



cargo install finlang



Check a program:



finlang check program.fin



Compile a program:



finlang compile program.fin



Run a program:



finlang run program.fin



Format a program:



finlang format program.fin



Run the project tests:



finlang test



⸻



32\. Language Pipeline



A FinLang program passes through the following conceptual pipeline:



Source

&#x20; ↓

Lexer

&#x20; ↓

Parser

&#x20; ↓

AST

&#x20; ↓

Type Checker

&#x20; ↓

Typed AST

&#x20; ↓

Compiler

&#x20; ↓

Bytecode

&#x20; ↓

Bytecode VM

&#x20; ↓

Financial State



The type checker enforces the language’s static rules before execution.



The VM enforces runtime preconditions and financial state invariants during execution.



⸻



33\. Current Language Scope



FinLang finlang-0.1 is a specialized financial DSL.



The current implementation is intentionally limited and does not attempt to provide the complete feature set of a general-purpose programming language.



Its current focus is:



\* deterministic execution

\* typed financial expressions

\* oracle verification

\* executable policies

\* financial state transitions

\* loan lifecycle operations

\* linear resource safety

\* bytecode execution

\* runtime verification



Additional language features may be introduced in future versions.

