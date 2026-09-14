# integrase (Python SDK)

Genetic design with the error bars attached.

Integrase operates on sequence data. It does not edit genomes, does not interact
with living organisms, and does not order DNA. Predictions are model outputs,
not measurements, and their intervals describe uncertainty about the model's
prediction rather than about what will happen in a cell.

```python
import integrase as ig

pred = ig.Prediction(...)
pred.value  # the point estimate
pred.interval  # Interval(lower=..., upper=...)
pred.coverage  # 0.80
pred.domain  # is the model guessing?
float(pred)  # TypeError, deliberately
```

That last line is the point. The one-liner that discards uncertainty is
`float(predict(design))`, and refusing it costs three seconds once.

Early stage: this package currently defines the surface. See the repository
root for the phased plan.
