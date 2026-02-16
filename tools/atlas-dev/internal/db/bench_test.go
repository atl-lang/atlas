package db

import (
	"testing"
)

func BenchmarkGetPhase(b *testing.B) {
	// Create in-memory database for benchmark
	db, err := New(":memory:")
	if err != nil {
		b.Fatal(err)
	}
	defer db.Close()

	if err := db.InitSchema(); err != nil {
		b.Fatal(err)
	}

	// Seed test phase
	result, err := db.Exec("INSERT INTO phases (path, name, category, status) VALUES ('bench.md', 'bench', 'test', 'pending')")
	if err != nil {
		b.Fatal(err)
	}
	id, _ := result.LastInsertId()

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		_, err := db.GetPhase(int(id))
		if err != nil {
			b.Fatal(err)
		}
	}
}

func BenchmarkGetCategory(b *testing.B) {
	db, err := New(":memory:")
	if err != nil {
		b.Fatal(err)
	}
	defer db.Close()

	if err := db.InitSchema(); err != nil {
		b.Fatal(err)
	}

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		_, err := db.GetCategory("stdlib")
		if err != nil {
			b.Fatal(err)
		}
	}
}

func BenchmarkGetTotalProgress(b *testing.B) {
	db, err := New(":memory:")
	if err != nil {
		b.Fatal(err)
	}
	defer db.Close()

	if err := db.InitSchema(); err != nil {
		b.Fatal(err)
	}

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		_, err := db.GetTotalProgress()
		if err != nil {
			b.Fatal(err)
		}
	}
}

func BenchmarkTransaction(b *testing.B) {
	db, err := New(":memory:")
	if err != nil {
		b.Fatal(err)
	}
	defer db.Close()

	if err := db.InitSchema(); err != nil {
		b.Fatal(err)
	}

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		err := db.WithTransaction(func(tx *Transaction) error {
			_, err := tx.Exec("SELECT 1")
			return err
		})
		if err != nil {
			b.Fatal(err)
		}
	}
}

func BenchmarkInsertAuditLog(b *testing.B) {
	db, err := New(":memory:")
	if err != nil {
		b.Fatal(err)
	}
	defer db.Close()

	if err := db.InitSchema(); err != nil {
		b.Fatal(err)
	}

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		err := db.InsertAuditLog("test", "phase", "1", "{}", "", "")
		if err != nil {
			b.Fatal(err)
		}
	}
}
