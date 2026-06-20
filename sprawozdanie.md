# Deska Galtona — sprawozdanie z projektu

Deska Galtona to proste urządzenie, które pokazuje skąd bierze się rozkład normalny. Kulka spada przez rzędy kołków — na każdym odbija się losowo w lewo lub w prawo. Po przejściu przez $n$ warstw trafia do jednego z pojemników na dole. Pojedyncza kulka jest nieprzewidywalna, ale gdy puścimy ich setki czy tysiące, histogram napełnienia pojemników rysuje charakterystyczną krzywą.

To jest właśnie **centralne twierdzenie graniczne** w akcji: suma wielu niezależnych, losowych odchyleń (każde odbicie od kołka) dąży do rozkładu normalnego, niezależnie od rozkładu pojedynczego zdarzenia.

## Fizyka w symulacji

### Rozkład normalny — predykcja napełnienia pojemników

Żeby z góry pokazać, na jakim poziomie powinny napełnić się pojemniki, program rysuje linię predykcji korzystając ze wzoru na **gęstość prawdopodobieństwa rozkładu normalnego**:

$$f(x) = \frac{1}{\sqrt{2\pi\sigma^2}} \cdot \exp\!\left(-\frac{(x - \mu)^2}{2\sigma^2}\right)$$

- $\mu$ — średnia, wyznacza środek rozkładu (środkowy pojemnik),
- $\sigma$ — odchylenie standardowe, mówi jak „szeroko" rozłożą się kulki,
- $x$ — numer pojemnika.

W kodzie to dosłowna implementacja tego wzoru:

```rust
fn normal_pdf(x: f32, mean: f32, stddev: f32) -> f32 {
    let var = stddev * stddev;
    let denom = (2.0 * PI * var).sqrt();
    let num = -(x - mean).powi(2) / (2.0 * var);
    (num.exp()) / denom
}
```

### Dobór odchylenia standardowego

W idealnej desce Galtona (czysto kombinatorycznej, bez fizyki) odchylenie standardowe wynosi $\sigma = \frac{\sqrt{n}}{2}$. Ale tutaj mamy pełną symulację fizyczną — kulki mają masę, tarcie, sprężystość — więc ich rozkład nie jest identyczny z modelem czysto probabilistycznym. Dlatego $\sigma$ jest dobrane **empirycznie**, tak żeby krzywa predykcji jak najlepiej pasowała do wyników symulacji:

$$\sigma = \frac{n}{6} + 1 - \ln\!\frac{n-8}{3}$$

```rust
let stddev_correction = 1. - ((n - 8) as f32 / 3.).ln();
let stddev = n as f32 / 6. + stddev_correction;
```

Człon logarytmiczny koryguje fakt, że przy większej liczbie warstw kulki mają więcej okazji do „ekstremalnych" odbić, co lekko poszerza rozkład.

### Współczynnik restytucji i tarcie

Każde zderzenie kulki z kołkiem podlega prawu odbicia z **współczynnikiem restytucji** $e$:

$$v_{\text{po}} = e \cdot v_{\text{przed}}$$

W symulacji $e = 0{,}5$ — kulka po odbiciu zachowuje połowę prędkości. To sprawia, że odbicia nie są idealnie sprężyste (jak w prawdziwym świecie) i kulki stopniowo tracą energię. Dodatkowo tarcie ($\mu_f = 0{,}05$) zapobiega nieskończonemu ślizganiu się.

```rust
const BALL_RESTITUTION: Restitution = Restitution::coefficient(0.5);
const BALL_FRICTION: Friction = Friction::coefficient(0.05);
```

Te wartości bezpośrednio wpływają na kształt końcowego rozkładu — zbyt sprężyste odbicia dałyby szerszy rozkład, zbyt tłumiące — węższy.

### Szacowanie wysokości pojemników

Program musi z góry wiedzieć, jak głębokie zrobić pojemniki, żeby zmieściły wszystkie kulki. Robi to tak:

1. Ze wzoru Gaussa wyznacza, jaki ułamek kulek trafi do najliczniejszego pojemnika ($n_{\max}$).
2. Liczy, ile miejsca te kulki zajmą, uwzględniając **współczynnik losowego upakowania kół** $c = 0{,}75$ (w 2D koła zapakowane losowo zajmują ~75% dostępnej powierzchni):

$$h = \frac{n_{\max} \cdot \pi r^2}{c \cdot w}$$

```rust
let ball_volume = PI * BALL_RADIUS * BALL_RADIUS;   // pole koła
let circle_packing = 0.75;
let required_area = max_count * ball_volume / circle_packing;
let required_bucket_height = required_area / bucket_width;
```

## Wnioski

Symulacja potwierdza centralne twierdzenie graniczne: nawet z pełną, realistyczną fizyką (sprężystość, tarcie, kolizje między kulkami) histogram rozkładu kulek w pojemnikach przyjmuje kształt krzywej Gaussa. Linie predykcji generowane wzorem na rozkład normalny dobrze pokrywają się z wynikami symulacji.
