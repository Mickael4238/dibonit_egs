flowchart TD
    %% Condition Nodes
    subgraph ConditionNodes["Condition Nodes"]
        A[("substr(A.B, 1) == 'A'")]
        B[("A.C != 'D'")]
    end

    %% Logical Node
    C{{"LOGICAL (OR)"}}

    %% Operator Nodes
    D["OPERATOR (== != > < >= <=)"]

    %% Function Nodes
    E["FUNCTION (getValue, matchKey)"]

    %% Value Nodes
    F["VALUE ('A', 1, 'D')"]

    %% Connections
    A -->|Condition| C
    B -->|Condition| C
    C -->|Uses| D
    D -->|Evaluates| E
    E -->|Returns| F
