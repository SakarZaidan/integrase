# The sandbox

A teaching simulator. Separate from the design tool, sharing its sequence core.

Nothing here is a model of a real organism. The genes are invented, most
parameters were chosen to make a lesson behave a particular way, and every one
of those says so when you ask it. Numbers from the sandbox are not predictions
about anything.

## The shape of it

One state object, many processes that read and change it, and a scheduler that
steps them. That is the architecture of the Karr whole-cell model, at far lower
fidelity: 28 submodels sharing a cell, generalised down to four.

```python
from integrase_sandbox.organism import default_processes, reference_cell
from integrase_sandbox.simulator import Simulator

sim = Simulator(reference_cell(), default_processes())
trajectory = sim.run(duration=600, dt=1.0, record_every=10)

print(trajectory.series("protein", "reporter"))
```

## Writing a process

A process is a class with a `step` method and a `name`. There is nothing to
inherit, nothing to register, and no `super().__init__()` to remember.

```python
class Inducer:
    """Adds a molecule when the clock passes a threshold."""

    @property
    def name(self) -> str:
        return "inducer"

    def step(self, state, dt):
        if state.time > 300:
            state.environment.inducers.add("IPTG", 0.1 * dt)

sim.add(Inducer())
```

That is the whole extension mechanism, and it is deliberately the whole of it.
A base class would put a barrier in front of the single most useful exercise
here.

Processes run in the order they were listed, and that order changes the numbers.
That is not a flaw being hidden: it is a real property of stepping continuous
processes discretely. Reorder `default_processes()` and watch the curves move.

## Where the numbers come from

Every constant is a `Parameter`, not a float:

```python
>>> from integrase_sandbox.processes import Transcription
>>> Transcription().rate.describe()
'0.05 transcripts/s per unit promoter strength [illustrative: chosen for teaching, not measured (gives a visible mRNA pool in ~1 min)]'
```

Three provenances exist. `MEASURED` came from a published measurement and names
it. `DERIVED` was computed from measured values and says how. `ILLUSTRATIVE` was
chosen so a teaching example behaves a particular way, and is not a number about
any organism.

A measured or derived parameter with no source is refused at construction. An
unsourced number is indistinguishable from a guess, and labelling one
"measured" would make it worse than an honest guess rather than better.

## Traits, and why they come back as odds

A trait takes a genome and an environment and returns a distribution, never a
single answer. For most traits a single answer would be false.

```python
from integrase_sandbox.conditions import EYE_COLOUR, eye_colour_genome
from integrase_sandbox.state import Environment

print(EYE_COLOUR.evaluate(eye_colour_genome(herc2_variant="A"), Environment()).describe())
```

```
eye colour:
   79.5%  blue
   16.1%  brown
    4.4%  green or hazel

What went into this:
  - Starting from the population baseline: 30% blue, 55% brown, 15% green or hazel
  - HERC2 position 12 carries A, shifting blue by +2.20 log odds (rs12913832 is
    the single largest known contributor. It is not in a pigment gene at all: it
    sits in a regulatory region of HERC2 and controls how much OCA2 is expressed)
```

Two lessons ship, chosen because they teach opposite things.

**Sickle cell anaemia** is the clean case. One gene, one base, and a mechanism
you can follow the whole way. The sixth codon of the mature haemoglobin beta
protein is GAG, coding for glutamate. Change the middle A to a T and it becomes
GTG, coding for valine. Glutamate is negatively charged and hydrophilic; valine
is uncharged and strongly hydrophobic. That puts a sticky patch on the surface
of the protein, and under low oxygen the molecules link into fibres that pull
the red cell out of shape.

The sandbox derives that explanation from an amino acid property table rather
than reciting it, so the same reasoning works for a substitution you invent.

**Eye colour is the correction.** The brown-dominant, blue-recessive model
taught in most schools is not a simplification. It is wrong. Roughly 16 genes
contribute, and the largest single contributor, rs12913832, is not in a pigment
gene at all: it sits in a regulatory region of *HERC2* and controls how much
*OCA2* is expressed. Two blue-eyed parents having a brown-eyed child is uncommon
and entirely possible, which the school model says cannot happen.

So eye colour ships as shifted odds with its reasoning shown, and a test asserts
that **no combination of the modelled variants ever reaches 95 percent**. If one
did, someone had tuned an effect size until a demo looked convincing, and the
lesson would have inverted into the model it exists to correct.

Writing your own trait works like writing your own process: a class with an
`evaluate` method and a `name`, and nothing to inherit.

## What it simplifies, and where

Stated here rather than left to be discovered.

**Counts, not molecules.** A real cell holds billions of each species. One entry
with a number is how every real simulator works, and it is the difference
between this running and not.

**One energy currency.** `ATP` stands in for ATP, GTP, NADH and the rest.

**First-order rates.** Production is proportional to input. Real transcription
saturates and real ribosomes compete for messages; neither happens here.

**No polymerase or ribosome pool.** Two genes with equal promoter strength
transcribe at equal rates however many other genes are running. In a cell they
would compete.

**Nutrients are one number between 0 and 1.** It stands in for every carbon and
nitrogen source.

**One copy of each gene.** Real genomes are diploid. Sickle cell anaemia is
recessive, so one variant copy gives sickle cell trait and two give the disease,
and the sandbox can only model the second. It says so where the trait is
reported rather than leaving the distinction out.

**Fixed time steps.** An adaptive integrator would be more accurate and would
make the timeline unpredictable. "One step is one second" needs to stay true for
someone watching, and the error from a fixed step is far smaller than the error
already in the parameters.

That last one is measurable, and worth measuring. Message levels settle at 2.45
where the analytic answer is 2.5, because transcription runs before degradation
in the same step and the message made this step decays this step. Halve `dt` and
the gap halves, which is what a first-order scheme should do.

## What it is not

Not a whole-cell model. Karr's took a large team years for the simplest organism
known, and this uses the same pattern at a fraction of the fidelity.

Not a drug discovery tool. Not a source of results. Not connected to the
biosecurity screening the design tool routes through, because nothing in the
sandbox can become physical DNA.
