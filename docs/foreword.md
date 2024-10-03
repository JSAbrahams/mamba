⬅ [🏠 Home](README.md)

# 0 📖 Foreword

Hello dear reader.
This document serves as an introduction to the Mamba language, or, a hobby project that got a bit out of hand. 
Whilst designing the language, I made the following rule for myself:

> A programming language should, ideally, make it easy to write clear descriptive unambiguous code, and difficult to 
write code that isn't those things

Another thing which I tried to keep in mind, which I think bears repeating, is the following:

> Code is more often read than it is written

In short, I wanted to create a programming language that is not only easy to write in, but also easy to read. 
There's this thing called the "no free lunch theorem", which in this context basically means that there is never a perfect solution to a problem.
Each potential solution comes with its own set of both advantage and disadvantages. 
It is up to the individual, or team, to weigh these trade-offs and make a, hopefully, well-informed decision. 
This also holds true for language design. 
There are a near endless amount of design choices, each of which come with their own set of advantages and disadvantages.
Below I present some examples of such trade-offs, and the accompanying thought processes.

- Increased flexibility of the language should make it easier for a developer to express their ideas in the language.
- At the same time, increased flexibility might mean there are more ways to do the same thing, which may lead to inconsistencies.
- Static typing allow us to identify errors which would otherwise only be raised during execution, when it might already be too late.
- However, enforcing types statically can potentially decrease flexibility and increase the verbosity of the language.
- Explicit error handling makes developers more aware of what might go wrong and what will happen in those situations.
- The increased verbosity of error handling mechanisms can however detract somewhat from the elegance of the language, and make code harder to read.

The list just keeps on growing.
Language design is tricky to say the least. 
I often see discussions online about what the supposed "best" programming language is. 
Personally, I think this misses the point wholly. 
A language, really, is just a tool to solve a certain set of problems. 
Just like tools in a toolbox, certain languages are better suited for certain jobs. 
I could tighten a screw with a hammer, but if I end up failing, that isn't the fault of the hammer, that's on me for picking the wrong tool. 
Were I to construct a mathematical model, languages such as python and MATLAB are probably more well-suited. 
If I wanted to create an embedded system, I would favour more low-level languages such as C or Rust, generally speaking.

Furthermore, the problems being solved also change over time. 
That is why I dismiss the notion of a "perfect programming language". 
A language, generally speaking, is designed to solve a certain set of problems well.
There will always be something that a certain language, or tool, is not design to solve. 
It stands to reason that the computer scientists of the 70's and 80's couldn't have imagined that computers would become as ubiquitous as they are today.
Machine learning has taken a foothold in most major industries, and the internet of things is starting to take off. 

I will say that certain languages features, no matter how well intentioned they may have been, or insignificant they seemed at the time, may not have been the best choice in hindsight. 
Not performing bounds checking on array accesses can result in buffer overflows, leading to many security exploits.
Having the concept of null in a language without null safety can result in undefined behaviour, and also allows developers to circumvent the type system of a language, significantly reducing its effectiveness.
However, hindsight is 20/20 as they say, and it is almost impossible to know how a language, or tools in general, will be used in advance. 
Truth be told, it is almost certain that any tool will be abused in ways the creator could not have foreseen. 
As the popular saying goes:

> A tool is either not used, or complained about

Still, over time, I think that we have gotten better at language design. 
Common patterns might be taken into account when designing a language, allowing for more elegant and idiomatic solutions to common problems. 
This holds especially for domain specific languages. 
I also think that it is possible to discourage, or make impossible, certain common mistakes made through language design.
For instance, making it impossible to return null values within methods unless explicitly stated that it may do so.

Personally, I think that when talking about general purpose languages (as opposed to more domain specific languages), a certain amount of restraint is necessary with regards to introducing features.
Too many features can result in a very steep learning curve, and might result in many possible ways of doing the same thing.
This is something I also struggled with during the design phase of the language. 
Certain feature might allow very specific problems to be solved idiomatically, but at a point lines have to be drawn, or we run into some issues.

- If I add a language feature to solve problem A, why should I not add one so we can more efficiently solve problem B? What about problem C?
- Every new feature added to a language increases its complexity, making it more difficult to write and read.

No language is perfect, including Mamba. 
As stated before, I think that it is impossible to design a perfect language. 
How computers are used and what problems are being solved changes over time, and different domains have different requirements and challenges. 
Nevertheless, I always think that it's worth it to try to improve upon past decisions, and see where that might lead us.

Joël S. Abrahams, 2019
