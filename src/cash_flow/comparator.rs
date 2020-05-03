use std::collections::{BTreeMap, HashSet, btree_map};

use log::{Level, log};

use crate::currency::{Cash, MultiCurrencyCashAccount};
use crate::formatting::format_date;
use crate::types::Date;

pub struct CashAssetsComparator<'a> {
    iter: btree_map::Iter<'a, Date, MultiCurrencyCashAccount>,
    next: Option<(&'a Date, &'a MultiCurrencyCashAccount)>,
    currencies: HashSet<&'static str>,
}

impl<'a> CashAssetsComparator<'a> {
    pub fn new(historical: &'a BTreeMap<Date, MultiCurrencyCashAccount>) -> CashAssetsComparator<'a> {
        let mut comparator = CashAssetsComparator {
            iter: historical.iter(),
            next: None,
            currencies: HashSet::new(),
        };
        comparator.next();
        comparator
    }

    pub fn compare(&mut self, date: Date, calculated: &MultiCurrencyCashAccount) -> bool {
        let (&date, actual) = match self.next {
            Some(data) if *data.0 < date => {
                self.next();
                (data.0, data.1)
            },
            _ => return self.next.is_none(),
        };

        self.currencies.extend(actual.iter().map(|assets| assets.currency));
        self.currencies.extend(calculated.iter().map(|assets| assets.currency));
        let currencies = self.currencies();

        // FIXME(konishchev): HERE
        let mut reported = false;

        for &currency in &currencies {
            let calculated_amount = calculated.get(currency).unwrap_or_else(||
                Cash::new(currency, dec!(0)));

            let actual_amount = actual.get(currency).unwrap_or_else(||
                Cash::new(currency, dec!(0)));

            if calculated_amount == actual_amount {
                continue;
            }

            let level = if self.next.is_some() {
                Level::Debug
            } else {
                Level::Warn
            };

            if !reported {
                log!(level, "Calculation error for {}:", format_date(date));
                reported = true;
            }
            log!(level, "* {} vs {} ({})",
                 calculated_amount, actual_amount, calculated_amount.sub(actual_amount).unwrap());
        }

        self.next.is_none()
    }

    fn currencies(&self) -> Vec<&'static str> {
        let mut currencies = self.currencies.iter().copied().collect::<Vec<_>>();
        currencies.sort();
        currencies
    }

    fn next(&mut self) {
        self.next = self.iter.next();
    }
}