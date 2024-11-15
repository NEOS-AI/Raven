package vectordb

import "context"

type VectorIndex interface {
	Dump() error
	Insert(ctx context.Context, vector []float32, id uint64) error
	InsertBatch(ctx context.Context, vectors [][]float32, ids []uint64) error
	Delete(ctx context.Context, id ...uint64) error
	//TODO other essential functions!
}
