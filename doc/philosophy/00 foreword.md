# Foreword

Hello dear reader.
This document serves as an introduction to the Mamba language. 
Whilst designing the language, I made the following rule for myself:

> A programming language should, ideally, make it easy to write clear descriptive unambiguous code, and difficult to 
write code that isn't those things

Another thing which I tried to keep in mind, which I think bears repeating, is the following:

> Code is more often read than it is written

In short, I wanted to create a programming language that is not only easy to write, but also easy to read. 
There's this thing called the "no free lunch theorem", which in this context basically means that there is no perfect solution to any problem. 
Each potential advantage comes with its own set of disadvantages. 
It is up to the engineer to weigh these trade-offs and make a, hopefully, well-informed decision. 
This also holds true for language design. 
There are a near endless amount of design choices, each with advantages and disadvantages.
Below I present some examples of such trade-offs and their accompanying thought process:

* Increased flexibility of the language makes it easier for a developer to express their ideas in the language.
* At the same time, increased flexibility can make it harder to read for others as there suddenly are more ways to do the same thing.
* Strong static typing rules allow us to catch errors which would otherwise only be raised during execution, when it might be too late.
* But, enforcing types statically can potentially decrease flexibility and increase verbosity.
* Explicit error handling makes developers more aware of what might go wrong and what will happen in those situations.
* The increased verbosity of error handling however detracts somewhat from the elegance of the language.

The list just keeps on growing. Language design is tricky to say the least. 
I often see discussions online about what the supposed "best" programming language is. 
Personally, I think this misses the point wholly. 
A language, really, is just a tool to solve a certain set of problems. 
Just like tools in a toolbox, certain languages are better suited for certain jobs. 
I could tighten a screw with a hammer, but if I end up failing, that isn't the fault of the hammer, that's on me for picking the wrong tool. 
Were I to construct a mathematical model, languages such as python and MATLAB are more well-suited. 
If I wanted to create an embedded system, more low-level languages such as C or Rust or more suited, generally speaking.

Furthermore, the problems being solved also change over time. 
That is why I dismiss the notion of a "perfect programming language". 
A language is designed to solve a certain set of problems well, so there will always be something that a certain language, or tool, is not design to solve. 
I can't imagine that the computer scientists of the 70's and 80's could've imagined that computers would become as wide-spread as they are today. 
As of writing this, in most developed countries, computers have become ubiquitous. 
Everyone seems to have in their pockets what not too long ago would've been called a supercomputer. 
Machine learning has taken a foothold in most major industries, and the internet of things is starting to take off. 

I will say that certain languages features, no matter how well intentioned they may have been, or insignificant they seemed at the time, may not have been the best choice in hindsight. 
Lack of bounds checking can lead to buffer overflows, lack of null safety leads to undefined behaviour, and so on. 
But hindsight is 20/20 as they say, and it is almost impossible to know how a language will be used in advance. 
Truth be told, it is almost certain that any tool will be abused in ways the creator could not have foreseen. 
As the popular saying goes:

> A tool is either not used, or complained about

Still, over time, I think that we have gotten better at language design. 
Common patterns might be taken into account when designing a language, allowing for more elegant idiomatic solutions. 
This is especially true with domain specific languages. 
Better still, mistakes can be discouraged or simply be made obsolete through language features. 

Personally, when talking about general purpose languages (as opposed to more domain specific languages), I believe we should stave off the temptation to add too many language features. 
A certain balance should be kept, or the language can, to my mind, start to feel asymmetric. 
This is something I also struggled with during the design phase. 
Certain feature might allow very specific problems to be solved idiomatically, but at a point lines have to be drawn, or we run into some issues:

* If I add a language feature to solve problem A, why should I not add one so we can more efficiently solve problem B? What about problem C?
* Every new feature added to a language increases its complexity, making it more difficult to write and read.
* Every new feature added has the potential to add yet another way to solve the same problem, potentially making the language more complex.

No language is perfect, including this one. 
As stated before, I think that it is impossible to design a perfect language. 
People, problems, how and where computers are used, changes over time, and different domains have different requirements and challenges. 
Nevertheless, I always think that it's worth to try to improve upon past decisions, and see where that might lead us.

Joël Abrahams, 2019
