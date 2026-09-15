# Reading an Integrase prediction

The most important page in these docs.

Integrase's central claim is that it will not hand you a number without telling
you how much to trust it. That claim is worthless if you do not know how to read
what it hands back. If you finish this page believing the intervals are
biological confidence intervals, this page has failed and so has the tool.

---

## The four parts of a prediction

```python
pred = ig.predict_translation_rate(design)

pred.value       # 4200.0                      the point estimate
pred.interval    # Interval(1900.0, 9100.0)    where the truth probably is
pred.coverage    # 0.80                        how often "probably" means
pred.domain      # DomainVerdict(...)          has the model seen this before?
pred.model       # "rbs-thermo-v1"             which model said so
```

There is deliberately no way to get the value alone without asking for it.
`float(pred)` raises a `TypeError`, because `rate = float(predict(design))` is
the one line that silently discards everything else on this page.

---

## What the interval means

A **calibrated 80% interval** means this: across many predictions on data
resembling the calibration set, intervals constructed this way contain the true
measured value about 80% of the time.

Three things follow.

**It is a statement about the procedure, not about your specific prediction.**
Your particular interval either contains the truth or it does not. The 80% is the
long-run rate.

**It is conditional on your input resembling the calibration data.** That
condition is exactly what `pred.domain` reports, and it is the reason the two are
separate fields.

**It is about the measurement the model was fitted against**, not about the
biology. See the next section, which is the part people get wrong.

---

## What the interval is not

**It is not a biological confidence interval.**

The models are fitted against measured expression from specific experiments. The
interval describes how well the model reproduces that kind of measurement. It
says nothing about:

- genomic context and neighbouring transcription
- DNA supercoiling
- your host's physiology, growth rate, or stress state
- media, temperature, or induction timing
- plasmid copy number
- metabolic burden from the construct itself

None of those are in the sequence, so no model reading only the sequence can
account for them. **The real spread of outcomes in your lab is wider than the
interval Integrase reports**, and how much wider depends on things the software
cannot see.

This is stated plainly because the alternative is a user treating an 80% interval
as an 80% chance their construct works, which it is not.

---

## The domain verdict

Separate from the interval, and the reason is technical rather than stylistic.

Calibration uses **split conformal prediction**, which gives a coverage guarantee
under an assumption called *exchangeability*: your input has to be, loosely, the
same kind of thing as the calibration data. When it is not, the guarantee does
not weaken gracefully. It stops applying.

So the guarantee evaporates precisely where you most need it, on the novel
sequence you actually wanted to ask about. Reporting one widened interval would
hide that. Two fields report two different things:

| Field | Question it answers |
|---|---|
| `interval` | Given this input is like my calibration data, how wrong am I likely to be? |
| `domain` | Is this input like my calibration data at all? |

### The four verdicts

**`InDistribution`**: your sequence resembles the validation set. The interval
carries its stated coverage. This is the case where the number means what it
says.

**`Marginal`**: at the edge. Something specific is unusual: a non-canonical
start codon, an unusual spacer length, strong predicted secondary structure. The
interval is reported and widened, and the reason names the trigger.

**`OutOfDistribution`**: outside anything the model was validated on, most often
a different host organism. **A value is still returned**, because refusing to
predict anything unfamiliar would make the tool useless for novel design, which
is what this field is for. But the stated coverage does not hold. Treat it as a
guess with a plausible magnitude.

**`Uncalibrated`**: this model has no calibration evidence at all. **No interval
is returned**, because a model that has never demonstrated coverage has no honest
interval to offer, and a wide one would imply a quantified uncertainty it has not
earned. Absence is the truthful answer.

---

## How to use this

**Rank by the lower bound, not the point estimate.** If you are choosing among
RBS variants and want the one most likely to clear a threshold, the conservative
choice is the highest lower bound, not the highest point estimate. A high
estimate with a wide interval is a weaker bet than a slightly lower one with a
tight interval.

```python
ranked = sorted(preds, key=lambda p: p.interval.lower, reverse=True)
```

**Filter on the domain verdict before trusting a ranking at all.**

```python
guessing = [p for p in preds if p.domain.is_out_of_distribution]
```

Those are the ones where the model has nothing to go on. They may still be your
most interesting designs. They are not ones to rank by number.

**Treat a wide interval as information, not noise.** When you change an RBS and
the interval widens, the model is telling you it understands the new sequence
less well. That is worth knowing before you order anything.

**Validate experimentally.** Nothing here substitutes for measurement. The point
of reporting uncertainty is to help you decide what to measure, not to avoid
measuring.

---

## Where the numbers come from

Every model carries a **model card**: published accuracy, which datasets it was
fitted and calibrated on, its known failure modes, and whether its coverage has
actually been demonstrated on held-out data.

```python
pred.model            # "rbs-thermo-v1"
```

Model cards are generated from the code that makes the claims, not written
alongside it, so the documentation cannot drift from the model. A model whose
card carries no measured coverage reports `Uncalibrated` and is not permitted to
claim a coverage figure.

---

## Further reading

- [`docs/limitations.md`](../limitations.md), the complete list of what this tool
  cannot do
- [`planning/subplans/16-biophysical-modeling-accuracy.md`](../../planning/subplans/16-biophysical-modeling-accuracy.md),
  the honest deep-dive on model accuracy and its failure modes
- [`planning/subplans/08-ml-architecture.md`](../../planning/subplans/08-ml-architecture.md),
  how calibration is produced and tested
