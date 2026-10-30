// Créé uniquement lors de la première initialisation du volume MongoDB.
// Le backend utilise un compte limité à sa base et non le compte racine.
db = db.getSiblingDB("qomune");
db.createUser({
  user: "qomune",
  pwd: process.env.QOMUNE_DB_PASSWORD,
  roles: [{ role: "readWrite", db: "qomune" }],
});
