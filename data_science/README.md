# Roman Mining Sites Analysis

## Overview

**Educational Data Science Project**: Analysis of 1,399 Roman mining sites from the **Oxford Roman Economy Project (OXREP) Database** using machine learning and statistical methods. This project demonstrates fundamental data science skills including EDA, data cleaning, visualization, and classification modeling as part of my learning journey.

**Data Source:** [OXREP Mines Database](https://oxrep.web.ox.ac.uk/mines-database)  

**Learning Objectives**: Practice CRISP-DM framework, handle messy data, build classification models, and communicate findings effectively.

---

## Research Questions

This project answers four core questions using CRISP-DM methodology:

### Questions 1-3: Descriptive & Inferential Statistics

1. **Which regions had the most Roman mining activity?**
   - Method: Descriptive statistics and visualization
   - Result: Spain dominated with 929 sites (66.4%), followed by Romania and Portugal
   - Insight: Iberian Peninsula was Rome's primary mining center

2. **What mining techniques were most commonly used?**
   - Method: Descriptive statistics and visualization
   - Result: Opencast (482 sites) and hydraulic mining (465 sites) were most prevalent
   - Insight: Romans preferred surface extraction and utilized advanced water-based techniques

3. **How complex were Roman mining operations?**
   - Method: Descriptive statistics and visualization
   - Result: 79.1% single-metal specialization, 19.7% polymetallic operations
   - Insight: Romans preferred focused, specialized operations

### Question 4: Machine Learning

4. **For specialized single-metal operations, can we predict which metal was extracted?**
   - Method: Machine Learning (Random Forest Classification)
   - Dataset: 1,106 single-metal specialized sites (79.1% of all Roman operations)
   - Classes: 8 metal types (Gold, Copper, Iron, Lead, Tin, Silver, MercuryCinnabar, Zinc)
   - Result: 99.55% accuracy achieved
   - Insight: Site characteristics strongly predict metal type, demonstrating systematic Roman site selection

---

## Results Summary

| Question | Method | Key Result |
|----------|--------|------------|
| 1. Which regions had most activity? | Descriptive stats | Spain=929 sites (66.4%), Romania=136, Portugal=72 |
| 2. What techniques were used? | Descriptive stats | Opencast=482 sites, Hydraulic=465, Underground=281 |
| 3. How complex were operations? | Descriptive stats | 79.1% single-metal, 19.7% polymetallic; Mean=1.24 metals/site |
| 4. Can we predict metal type for specialized sites? | ML Classification | 99.55% accuracy; 8 metal classes; 1,106 single-metal sites |

---

## Key Findings

**Geographic Distribution:**
- Spain dominated with 929 sites (66.4% of total Roman mining operations)
- Iberian Peninsula was the primary Roman mining center

**Technology & Techniques:**
- Opencast mining (482 sites) and hydraulic mining (465 sites) most common
- Romans preferred surface extraction over underground methods

**Operational Strategy:**
- 79.1% of sites specialized in single-metal extraction
- 19.7% extracted multiple metals (polymetallic operations)  
- Mean of 1.24 metals per site indicates high specialization

**Predictive Modeling:**
- 99.55% classification accuracy on 1,106 single-metal specialized sites
- Multi-class prediction across 8 metal types (Gold, Copper, Iron, Lead, Tin, Silver, MercuryCinnabar, Zinc)
- Only 1 misclassification out of 222 test samples (Zinc misclassified as Silver)
- Dataset distribution: Gold (72%), Copper (16%), Iron (6%), others (6%)

---

## Technologies

**Python Libraries:**
- **Data Processing:** pandas, numpy
- **Visualization:** matplotlib
- **Machine Learning:** scikit-learn (RandomForestClassifier, train_test_split, classification_report, confusion_matrix, accuracy_score)

**Environment:** Python 3.7+, Jupyter Notebook

---

## Project Files

```
project-data-science/
├── Roman_Mining_EDA_Analysis.ipynb  # Main analysis (38 cells with helper functions)
├── oxrep-mines-3.0-20250408.csv     # Data (1,399 sites × 47 features)
└── README.md                         # This file (technical overview)
```

**Notebook Structure:**
1. **Business Understanding** - 4 research questions defined
2. **Data Understanding** - Initial exploration + helper functions (DRY principles)
3. **Data Preparation** - Cleaning and transformation using helper functions
4. **Modeling** - Random Forest classification (Question 4)
5. **Evaluation** - Model assessment and descriptive analytics (Questions 1-3)
6. **Deployment** - Summary of findings

---

## Documentation

- **README.md** (this file) - Technical project overview with results summary

---

## How to Run

**Installation:**
```bash
# Create virtual environment
python3 -m venv .venv
source .venv/bin/activate

# Install dependencies
pip install pandas numpy matplotlib scikit-learn jupyter
```

**Execution:**
```bash
# Start Jupyter
jupyter notebook

# Open Roman_Mining_EDA_Analysis.ipynb
# Run cells sequentially (Cell > Run All)
```

**Requirements:**
- Data file `oxrep-mines-3.0-20250408.csv` in same directory
- Python 3.7+ recommended
- Runtime: ~30-60 seconds

---

## Acknowledgements

**Data:** [Oxford Roman Economy Project (OXREP)](https://oxrep.web.ox.ac.uk/mines-database)  
**Framework:** CRISP-DM methodology  
**Tools:** Python, scikit-learn, pandas, Jupyter  
**Learning Context:** Created as an educational data science project

---

## License & Usage

**Educational Project** - Created for learning purposes and portfolio demonstration  
**Data:** OXREP Mining Database - see [terms of use](https://oxrep.web.ox.ac.uk/mines-database)  
**Code:** Free to use with attribution for educational purposes

---
