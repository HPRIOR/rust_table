# 'Table' data structure test written in rust

A POC for a 'table' data structure for an ECS system. The table is essentially a 2D array where each 'column' can be any type. The types of each column cannot be known at compile time so generics can't be used. The arrays must also be cache friendly, so no indirection and dynamic types either. The current implementaiton retrieves type information of the provided column types to construct an array of arrays for each provided type (using unsafe rust, memory layouts, allocation api etc). 
